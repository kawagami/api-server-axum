use crate::{
    repositories::torrents as torrents_repo,
    state::AppState,
    structs::{
        torrents::{Torrent, TorrentFile},
        ws::WsEvent,
    },
};
use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, ManagedTorrent};
use std::{sync::Arc, time::Duration};
use super::manager::{DEFAULT_METADATA_TIMEOUT_SECONDS, INIT_TIMEOUT, MAX_METADATA_ATTEMPTS, POLL_INTERVAL, Slot, max_active, setting};
use super::session::{purge_by_info_hash, remove_from_session};

/// 啟動失敗的分類 —— 讓位還有重試機會，其他錯誤直接判 failed
enum StartFailure {
    /// 檢查點到了還沒找到 peers，而且有任務排隊等名額 → 讓位（額度未用完只是排到隊尾）
    MetadataTimeout,
    Fatal(String),
}

/// 把排隊中（pending）與重啟後中斷（downloading）的任務補進 session，直到達併發上限。
/// 啟動時、新增後、完成/失敗/讓位/刪除後都會呼叫；重複呼叫安全。
/// 回傳 BoxFuture：本函式 spawn `run_torrent`，而它收尾時又回頭 spawn 本函式，
/// opaque future 會變成型別自我參照（不 Send）
pub fn sync_active(state: AppState) -> futures::future::BoxFuture<'static, ()> {
    Box::pin(sync_active_inner(state))
}

async fn sync_active_inner(state: AppState) {
    let manager = state.get_torrents();
    let max_active = max_active(&state);

    let mut slots = manager.active.lock().await;
    if slots.len() >= max_active {
        return;
    }
    // limit 要把已佔位的算進去 —— 它們也在 resumable 清單裡而且排在前面，
    // 只撈 max_active 筆會被自己佔滿、撈不到後面排隊的
    let rows = match torrents_repo::list_resumable(
        state.get_pool(),
        (max_active + slots.len()) as i64,
    )
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("sync_active db error: {e}");
            return;
        }
    };
    let free = max_active - slots.len();
    let to_start: Vec<Torrent> = rows
        .into_iter()
        .filter(|t| !slots.contains_key(&t.id))
        .take(free)
        .collect();

    // 各自 spawn，不在這裡逐一 await：add_torrent 會等 magnet metadata，
    // 序列跑的話第一筆沒 peers 就把後面全卡死
    for row in to_start {
        let id = row.id;
        let task = tokio::spawn(run_torrent(state.clone(), row));
        slots.insert(id, Slot { task, handle: None });
    }
}

/// 有沒有任務排不進併發名額。
/// 這是「解析 metadata 該不該讓位」的唯一判準 —— 沒人在等就沒有理由中斷冷門種子的解析。
/// 查詢失敗時保守回 true（維持讓位行為，寧可多輪替也別卡住整條隊伍）。
async fn queue_pressure(state: &AppState) -> bool {
    match torrents_repo::count_resumable(state.get_pool()).await {
        Ok(total) => total > max_active(state) as i64,
        Err(e) => {
            tracing::warn!("count_resumable failed: {e}");
            true
        }
    }
}

/// 一個任務的完整生命週期：加進 session（解析 metadata，中途可能讓位）→ 監看到完成/失敗。
/// 由 `sync_active` spawn，對應 active map 裡的一格。
async fn run_torrent(state: AppState, row: Torrent) {
    let id = row.id;
    // 記一次嘗試：把自己推到候選排序的隊尾，這輪沒成功時後面的任務才輪得到。
    // DB 掛掉時當第 1 次 —— 寧可多試幾輪，也不要因為記不到帳就把任務判失敗
    let attempt = torrents_repo::mark_attempt(state.get_pool(), id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("torrent {id} mark_attempt failed: {e}");
            1
        });

    let failure = match start_torrent(&state, &row).await {
        // 成功後 watcher 自己收尾（完成/失敗都會清 slot 並補位）
        Ok(handle) => return watch_torrent(state, id, handle).await,
        Err(failure) => failure,
    };

    // 讓位（還有額度）→ 留在 pending 等下一輪；其餘一律判 failed。
    // 兩條路的收尾動作相同：清 slot → 更新 DB → 推播 → 補位
    let retrying =
        matches!(failure, StartFailure::MetadataTimeout) && attempt < MAX_METADATA_ATTEMPTS;
    let reason = match failure {
        StartFailure::MetadataTimeout if retrying => {
            format!("找不到 peers，先讓位給排隊的任務（第 {attempt}/{MAX_METADATA_ATTEMPTS} 次）")
        }
        StartFailure::MetadataTimeout => {
            format!("找不到 peers，連續 {MAX_METADATA_ATTEMPTS} 次讓位後仍無結果")
        }
        StartFailure::Fatal(reason) => reason,
    };

    state.get_torrents().active.lock().await.remove(&id);
    // 這兩個是狀態機轉換（不是快取寫入），失敗不能只用 `let _ =` 吞掉：任務會卡在
    // downloading，list_resumable 一直把它撈出來重試，但 attempt_count 沒遞增，
    // 於是永遠不會被判失敗 —— 變成無限重試迴圈。至少要留下可查的 error log。
    if retrying {
        tracing::warn!("torrent {id} {reason}");
        if let Err(e) = torrents_repo::set_retry_pending(state.get_pool(), id, &reason).await {
            tracing::error!("torrent {id} set_retry_pending failed: {e}（狀態可能卡在 downloading）");
        }
        state.broadcast(
            WsEvent::TorrentRetrying,
            serde_json::json!({ "id": id, "name": row.name, "reason": reason, "attempt": attempt }),
        );
    } else {
        tracing::error!("torrent {id} start failed: {reason}");
        if let Err(e) = torrents_repo::set_failed(state.get_pool(), id, &reason).await {
            tracing::error!("torrent {id} set_failed failed: {e}（狀態可能卡在 downloading）");
        }
        broadcast_failed(&state, id, row.name.as_deref(), &reason);
    }
    tokio::spawn(sync_active(state));
}

async fn start_torrent(
    state: &AppState,
    row: &Torrent,
) -> Result<Arc<ManagedTorrent>, StartFailure> {
    let manager = state.get_torrents();
    let output_dir = manager.output_dir(&row.info_hash);
    let interval = Duration::from_secs(
        setting(
            state,
            "torrent_metadata_timeout_seconds",
            DEFAULT_METADATA_TIMEOUT_SECONDS,
        )
        .clamp(30, 3600) as u64,
    );

    // 解析可能耗上數分鐘，這行是那段期間唯一的痕跡（started 要等解析完才印）
    tracing::info!(
        "torrent {} ({}) resolving metadata, check every {}s",
        row.id,
        row.info_hash,
        interval.as_secs()
    );

    // ⚠ 對 magnet，librqbit 會在 add_torrent 內部解析 metadata（DHT/tracker 找 peers 要 info），
    //   而且它自己沒有逾時 —— 種子沒人做種就是無限期卡在這個 await，任務永遠停在 pending。
    //   逾時必須包在這裡，包在後面的 wait_until_initialized 已經來不及。
    let add_fut = manager.session.add_torrent(
        AddTorrent::from_url(&row.magnet_uri),
        Some(AddTorrentOptions {
            // 重啟 resume：檔案已存在時驗證既有 piece 續抓，不整包重來
            overwrite: true,
            output_folder: Some(output_dir.to_string_lossy().to_string()),
            ..Default::default()
        }),
    );
    tokio::pin!(add_fut);

    // 到點只是「檢查要不要讓位」，不是硬逾時：沒有任務排隊就繼續等下一輪。
    // 對同一個 pinned future 反覆 timeout（而不是重新 add_torrent），已累積的
    // DHT 查詢與半握手的 peer 都留著 —— 冷門種子要的就是不被打斷的時間。
    let response = loop {
        match tokio::time::timeout(interval, &mut add_fut).await {
            Ok(Ok(r)) => break r,
            Ok(Err(e)) => return Err(StartFailure::Fatal(format!("add_torrent failed: {e}"))),
            Err(_) => {
                if !queue_pressure(state).await {
                    tracing::info!(
                        "torrent {} 仍在解析 metadata，無任務排隊等名額，繼續等待",
                        row.id
                    );
                    continue;
                }
                // 有任務排不進名額 → 讓位，本輪放棄（逾時當下極小機率剛好已掛進
                // session，用 info_hash 補刪，別留孤兒）
                purge_by_info_hash(manager, &row.info_hash, false).await;
                return Err(StartFailure::MetadataTimeout);
            }
        }
    };

    let handle = match response {
        AddTorrentResponse::Added(_, handle) => handle,
        AddTorrentResponse::AlreadyManaged(_, handle) => handle,
        AddTorrentResponse::ListOnly(_) => {
            return Err(StartFailure::Fatal(
                "unexpected list-only response".to_string(),
            ))
        }
    };

    // 佔位格補上 handle（即時進度要用）。格子不見 = 解析期間任務被刪掉了
    match manager.active.lock().await.get_mut(&row.id) {
        Some(slot) => slot.handle = Some(handle.clone()),
        None => {
            // 刪不掉就是 librqbit session 裡留了一個沒人管的 torrent（繼續佔頻寬與磁碟），
            // 而外層只會看到「任務已被移除」這個預期內的錯誤 —— 不記就查不出殘留從哪來
            if let Err(e) = manager.session.delete(handle.id().into(), true).await {
                tracing::warn!("torrent {} 佔位格消失後回收 session 失敗: {}", row.id, e);
            }
            return Err(StartFailure::Fatal(
                "任務已於啟動期間被移除".to_string(),
            ));
        }
    }
    tracing::info!("torrent {} ({}) started", row.id, row.info_hash);
    Ok(handle)
}

/// 拿到 metadata 之後的監看：等初始化 → metadata 落 DB → 進度推播 → 完成/失敗收尾
async fn watch_torrent(state: AppState, id: i32, handle: Arc<ManagedTorrent>) {
    // 1. 等初始化完成（metadata 已在 start_torrent 拿到，這裡等的是既有檔案的 piece 驗證）
    match tokio::time::timeout(INIT_TIMEOUT, handle.wait_until_initialized()).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            finish_failed(&state, id, &handle, &format!("初始化失敗: {e}")).await;
            return;
        }
        Err(_) => {
            finish_failed(&state, id, &handle, "初始化逾時（既有檔案驗證未完成）").await;
            return;
        }
    }

    // 2. metadata 落 DB
    let name = handle.name().unwrap_or_else(|| format!("torrent-{id}"));
    let files: Vec<TorrentFile> = match handle.with_metadata(|m| {
        m.file_infos
            .iter()
            .enumerate()
            .map(|(index, f)| TorrentFile {
                index,
                path: f.relative_filename.to_string_lossy().to_string(),
                size: f.len,
            })
            .collect()
    }) {
        Ok(files) => files,
        Err(e) => {
            finish_failed(&state, id, &handle, &format!("讀取 metadata 失敗: {e}")).await;
            return;
        }
    };
    let total_size: i64 = files.iter().map(|f| f.size as i64).sum();
    let files_json = match serde_json::to_value(&files) {
        Ok(v) => v,
        Err(e) => {
            finish_failed(&state, id, &handle, &format!("序列化檔案清單失敗: {e}")).await;
            return;
        }
    };
    if let Err(e) =
        torrents_repo::set_downloading_metadata(state.get_pool(), id, &name, total_size, &files_json)
            .await
    {
        tracing::error!("torrent {id} metadata db update failed: {e}");
    }

    // 3. 輪詢進度：5 秒一次、有變動才推播
    let mut last_percent = -1.0_f64;
    loop {
        let stats = handle.stats();

        if let Some(error) = stats.error {
            finish_failed(&state, id, &handle, &error).await;
            return;
        }

        if stats.finished {
            finish_completed(&state, id, &handle, &name, total_size).await;
            return;
        }

        let percent = if stats.total_bytes > 0 {
            (stats.progress_bytes as f64 / stats.total_bytes as f64 * 10000.0).round() / 100.0
        } else {
            0.0
        };
        if (percent - last_percent).abs() > f64::EPSILON {
            last_percent = percent;
            let (down_speed, peers) = stats
                .live
                .as_ref()
                .map(|l| (l.download_speed.to_string(), l.snapshot.peer_stats.live))
                .unwrap_or_default();
            state.broadcast(
                WsEvent::TorrentProgress,
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "progress": percent,
                    "progress_bytes": stats.progress_bytes,
                    "total_bytes": stats.total_bytes,
                    "down_speed": down_speed,
                    "peers": peers,
                }),
            );
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// 完成收尾：從 session 移除（停止做種、保留檔案）→ DB → 推播 → 補位
async fn finish_completed(
    state: &AppState,
    id: i32,
    handle: &Arc<ManagedTorrent>,
    name: &str,
    total_size: i64,
) {
    remove_from_session(state, id, handle, false).await;
    if let Err(e) = torrents_repo::set_completed(state.get_pool(), id).await {
        tracing::error!("torrent {id} set_completed failed: {e}");
    }
    tracing::info!("torrent {id} ({name}) completed");
    state.broadcast(
        WsEvent::TorrentCompleted,
        serde_json::json!({ "id": id, "name": name, "total_size": total_size }),
    );
    {
        let settings = state.get_settings();
        let subject = format!("Torrent 下載完成：{name}");
        let body = format!(
            "任務 #{id}\n名稱：{name}\n大小：{}\n\n到後台 /admin/torrents 產生下載連結取檔。",
            format_size(total_size)
        );
        tokio::spawn(async move {
            // 這封是**一次性**的：torrent 沒有「未通知」欄可以留著下輪補寄，寄失敗就是
            // 失敗（`send_to` 已記 ERROR）。使用者仍看得到任務變 completed，只是少一封信。
            let _ = crate::services::email::send_notification(&settings, &subject, body).await;
        });
    }
    // spawn 而非 await：斷開 watch_torrent ↔ sync_active 的遞迴，避免 future Send 自我參照
    tokio::spawn(sync_active(state.clone()));
}

fn format_size(bytes: i64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else {
        format!("{:.1} MB", b / MB)
    }
}

/// 失敗收尾：從 session 移除（保留已下載部分供重試續抓）→ DB → 推播 → 補位
async fn finish_failed(state: &AppState, id: i32, handle: &Arc<ManagedTorrent>, reason: &str) {
    remove_from_session(state, id, handle, false).await;
    let name = handle.name();
    if let Err(e) = torrents_repo::set_failed(state.get_pool(), id, reason).await {
        tracing::error!("torrent {id} set_failed failed: {e}");
    }
    tracing::warn!("torrent {id} failed: {reason}");
    broadcast_failed(state, id, name.as_deref(), reason);
    tokio::spawn(sync_active(state.clone()));
}

fn broadcast_failed(state: &AppState, id: i32, name: Option<&str>, reason: &str) {
    state.broadcast(
        WsEvent::TorrentFailed,
        serde_json::json!({ "id": id, "name": name, "reason": reason }),
    );
}

use crate::state::AppState;
use librqbit::{ManagedTorrent, Session, SessionOptions};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

/// metadata 解析的**讓位檢查間隔**預設值（秒）— 可由 app_settings.torrent_metadata_timeout_seconds
/// 熱更新（key 名沿用歷史，語意已不是硬逾時）。到點只檢查有沒有任務排隊等名額：
/// 沒有就繼續等（冷門種子需要時間），有才放棄本輪。
pub(super) const DEFAULT_METADATA_TIMEOUT_SECONDS: i64 = 180;
/// 讓位次數上限：被排隊任務擠掉這麼多次仍沒拿到 metadata 才判 failed。
/// 沒人排隊時不會累計，所以「一直等」不會用掉額度。
pub(super) const MAX_METADATA_ATTEMPTS: i32 = 3;
/// 初始化逾時 — metadata 到手後驗證磁碟既有 piece（純本地 IO + SHA-1，大檔在小機器上會久）
pub(super) const INIT_TIMEOUT: Duration = Duration::from_secs(1800);
/// 進度輪詢間隔
pub(super) const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// 下載連結效期預設值（分鐘）— 可由 app_settings.torrent_link_ttl_minutes 熱更新
pub(super) const DEFAULT_LINK_TTL_MINUTES: i64 = 180;

const DEFAULT_MAX_ACTIVE: usize = 2;
pub(super) const DEFAULT_MAX_TOTAL_SIZE_GB: i64 = 20;

/// 進行中任務的一格。
/// **佔位早於 `add_torrent`**：magnet 的 metadata 解析在 librqbit 內部進行、可能耗上數分鐘，
/// 那段期間還沒有 handle，但名額已經被吃掉，不先佔位就會被重複啟動、併發數也算不準。
pub(super) struct Slot {
    /// 整條生命週期的 task（啟動 + 監看），刪除時 abort
    pub(super) task: JoinHandle<()>,
    /// metadata 解析完才有；None = 還在解析
    pub(super) handle: Option<Arc<ManagedTorrent>>,
}

/// torrent session 與進行中任務的 handle 對照表。
/// 進度不落 DB — 即時資訊一律從 handle 讀。
pub struct TorrentManager {
    pub(super) session: Arc<Session>,
    pub(super) active: Mutex<HashMap<i32, Slot>>,
    base_path: PathBuf,
}

impl TorrentManager {
    pub async fn new() -> Self {
        let base_path =
            PathBuf::from(std::env::var("TORRENT_PATH").unwrap_or_else(|_| "./torrents".to_string()));
        tokio::fs::create_dir_all(&base_path)
            .await
            .expect("failed to create TORRENT_PATH");
        // `ipv4_only: true` —— 不能用 `Session::new`（它等於全默認）。
        //
        // librqbit 9 的招牌改動是「預設全走 dualstack IPv6 + IPv4」：
        // `SessionOptions::default()` 的 `ipv4_only` 是 false、`dht` 是 Some，於是 DHT
        // 的 UDP socket 綁 `[::]`、outgoing peer 連線也會去撥 v6 peer。
        // 而這台 VPS 從來沒有可用的 IPv6（無全域位址、無 `::/0`），宿主機已於
        // 2026-08-09 整台關掉 —— 理由見 deploy/README.md「主機 IPv6：必須整台關掉」。
        //
        // ⚙ 2026-08-23 升 9 時在 production 量過這三格（在 backend 的 netns 內）：
        //   /proc/sys/net/ipv6/conf/all/disable_ipv6 → 1
        //   /proc/net/if_inet6                       → 檔存在但內容全空（模組有載、零位址）
        //   bind(AF_INET6, "::", 0)                  → **成功**
        // 所以這行**不是在防 panic** —— wildcard 綁得起來，Session 建得起來。
        // 它防的是【靜默浪費】：DHT 學到的 v6 節點、tracker 回的 compact6 peer，
        // 每一發都撒進死巷，而 librqbit 的 target 不在 EnvFilter 的白名單上
        // （預設 `{crate}=info,tower_http=warn`，見 main.rs::default_log_filter）
        // → 它的 WARN 進不了 stdout 也進不了 logs 表，**你一行都看不到**。
        //
        // ⚠ `ipv4_only` **不是全覆蓋的**（librqbit 9.0.1 實測，數 /proc/self/fd 對
        // /proc/self/net/udp6 得到的結果，見本檔 tests）:
        //   DHT           → 吃 `ipv4_only`，綁 0.0.0.0 ✅
        //   outgoing peer → 吃 `ipv4_only` ✅
        //   LSD           → **不吃** —— session.rs 建它時只傳 `..Default::default()`，
        //                   `ipv4_only` 沒被轉進去，會多一個 `[::]:6771` 的 v6 多播 socket。
        //                   ⇒ 直接 `disable_local_service_discovery: true`:LSD 找的是
        //                   **同網段**的 peer，VPS 上沒有這種鄰居，本來就是純浪費。
        //   UDP tracker   → **不吃**，而且**改不了** —— tracker_comms_udp.rs 把 `[::]:0`
        //                   寫死。但 `BindOpts::default()` 是 `request_dualstack: true`
        //                   （→ `only_v6=0`），所以它靠 v4-mapped 走 v4 出去，功能正常。
        //                   代價:那顆 socket 是 AF_INET6。**這是脆弱點** —— 哪天 IPv6
        //                   模組真的不在（`ipv6.disable=1`），`socket()` 會回 EAFNOSUPPORT，
        //                   `Session::new*` 直接 Err → 這裡的 `.expect` 就會炸掉整個 backend。
        //                   現況安全的依據是上面量到的「模組有載、wildcard 綁得起來」。
        //
        // 哪天真的把 IPv6 設起來（provider 給 /64 + Docker 網路 enable_ipv6），
        // 這行跟 nginx 的 `listen [::]` 一起拉回來。
        let session = Session::new_with_opts(
            base_path.clone(),
            SessionOptions {
                ipv4_only: true,
                disable_local_service_discovery: true,
                ..Default::default()
            },
        )
        .await
        .expect("failed to create torrent session");
        Self {
            session,
            active: Mutex::new(HashMap::new()),
            base_path,
        }
    }

    pub fn base_path(&self) -> &std::path::Path {
        &self.base_path
    }

    pub fn output_dir(&self, info_hash: &str) -> PathBuf {
        self.base_path.join(info_hash)
    }

    pub async fn get_handle(&self, id: i32) -> Option<Arc<ManagedTorrent>> {
        self.active.lock().await.get(&id).and_then(|s| s.handle.clone())
    }
}

/// 讀 app_settings 的數值設定，缺失/壞值一律回退預設（型別由 default 決定）
pub(super) fn setting<T: std::str::FromStr>(state: &AppState, key: &str, default: T) -> T {
    state
        .get_settings()
        .get(key)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

/// 併發上限（正在解析 metadata 的佔位也算一格）
pub(super) fn max_active(state: &AppState) -> usize {
    setting(state, "torrent_max_active", DEFAULT_MAX_ACTIVE)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `TorrentManager::new()` 真的建得起來，而且 socket 真的是 IPv4 —— **`#[ignore]`，CI 不跑**。
    ///
    /// 為什麼要有這條:整條 torrent 路徑在 CI 是零覆蓋，而 `new()` 裡的
    /// `.expect("failed to create torrent session")` 在**啟動路徑**上 —— 它炸掉不是
    /// 「torrent 功能壞了」，是**整個 backend 開不起來**。dependabot 只會改版號，
    /// 而 librqbit 8→9 的預設就從 v4 變成 dualstack（見 `new()` 裡的長註解）。
    ///
    /// 為什麼不只斷言「不 panic」:librqbit 綁 `[::]` 在**開發機**上是會成功的（本機有
    /// IPv6），所以只驗 `new()` 回得來等於什麼都沒驗。要驗的是**綁出來的 family**。
    ///
    /// 為什麼 `#[ignore]`:這條會建真的 DHT socket 並往外 bootstrap。放進 CI 等於讓一個
    /// 網路依賴決定 test job 綠不綠。
    ///
    /// 升 librqbit 大版時在本機跑一次:
    /// ```text
    /// cargo test --locked -- --ignored torrent_session_builds --nocapture
    /// ```
    #[tokio::test]
    #[ignore = "會建真的 DHT socket 並對外 bootstrap;升 librqbit 大版時手動跑"]
    async fn torrent_session_builds() {
        let dir = std::env::temp_dir().join("kawa-torrent-session-test");
        // SAFETY: 單執行緒測試、只設一次，沒有其他 thread 同時讀 env
        unsafe { std::env::set_var("TORRENT_PATH", &dir) };
        let manager = TorrentManager::new().await;
        assert_eq!(manager.base_path(), dir.as_path());

        let v4 = own_udp_sockets("/proc/self/net/udp");
        let v6 = own_udp_sockets("/proc/self/net/udp6");
        // 失敗時看得到全貌，才不必重跑一次加 print
        println!("own udp4: {v4:#?}\nown udp6: {v6:#?}");

        // DHT 吃 `ipv4_only`，所以一定有一顆 v4 UDP socket。一顆都沒有 = 上游又改了預設。
        assert!(!v4.is_empty(), "沒有任何 v4 UDP socket，`ipv4_only` 疑似沒生效");

        // LSD 不吃 `ipv4_only`，靠 `disable_local_service_discovery` 關掉。
        // 6771 = BitTorrent LSD 的多播埠;它出現代表那個開關失效或上游改名。
        assert!(
            !v6.iter().any(|s| s.port == 6771),
            "LSD 還活著（[::]:6771），`disable_local_service_discovery` 沒生效"
        );

        // 剩下允許存在的 v6 socket 只有 UDP tracker client 那顆（`[::]:0` 寫死、dualstack，
        // 功能正常）。多於一顆就是又有新東西不吃 `ipv4_only`，回去讀 `new()` 的註解。
        assert!(
            v6.len() <= 1,
            "多出沒預期的 v6 socket，可能有新元件不吃 `ipv4_only`: {v6:#?}"
        );

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[derive(Debug)]
    struct OwnSocket {
        port: u16,
        /// 原始的 `hex_addr:hex_port`。只在 assert 失敗訊息（`Debug`）裡出現，
        /// dead code 分析看不到 `Debug` 的用法，所以要 `expect`。
        #[expect(dead_code)]
        local: String,
    }

    /// 本行程自己持有的 UDP socket。
    /// `/proc/self/net/udp*` 是**整個 netns** 的（會有別的行程），所以要跟
    /// `/proc/self/fd` 的 `socket:[inode]` 交叉比對才算是「自己的」。
    fn own_udp_sockets(table: &str) -> Vec<OwnSocket> {
        let mut own = std::collections::HashSet::new();
        for entry in std::fs::read_dir("/proc/self/fd")
            .expect("read /proc/self/fd")
            .flatten()
        {
            let inode = std::fs::read_link(entry.path()).ok().and_then(|target| {
                target
                    .to_string_lossy()
                    .strip_prefix("socket:[")
                    .and_then(|s| s.strip_suffix(']'))
                    .and_then(|s| s.parse::<u64>().ok())
            });
            if let Some(inode) = inode {
                own.insert(inode);
            }
        }

        std::fs::read_to_string(table)
            .unwrap_or_default()
            .lines()
            .skip(1) // 標頭
            .filter_map(|line| {
                let cols: Vec<&str> = line.split_whitespace().collect();
                // 欄位:1 = local_address（`hex_addr:hex_port`），9 = inode
                let local = cols.get(1)?;
                let inode: u64 = cols.get(9)?.parse().ok()?;
                if !own.contains(&inode) {
                    return None;
                }
                let port = u16::from_str_radix(local.split(':').nth(1)?, 16).ok()?;
                Some(OwnSocket {
                    port,
                    local: (*local).to_string(),
                })
            })
            .collect()
    }
}

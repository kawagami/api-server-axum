mod admin;
mod admin_blog_comments;
mod admin_games;
mod admin_gov_tenders;
mod admin_invoice_lottery;
mod admin_messages;
mod admin_stats;
mod admin_vocab;
mod app_settings;
mod admin_blogs;
mod audit_logs;
mod auth;
mod blogs;
mod logs;
mod images;
mod invoices;
mod ledger;
mod lotto;
mod members;
mod messages;
mod metrics;
mod oauth;
mod permissions;
mod portfolio;
mod roles;
mod roster;
mod stocks;
mod tools;
mod torrents;
mod users;
mod vocab;
mod ws;

use crate::extract::Json;
use crate::{
    errors::{AppError, RequestError},
    logging::LogEntry,
    scheduler::initialize_scheduler,
    state::AppState,
    structs::features::Feature,
};
use axum::{
    extract::{connect_info::ConnectInfo, DefaultBodyLimit, Request, State},
    http::{header, HeaderValue, Method},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    Router
};
use std::{net::SocketAddr, time::Duration};
use tokio::sync::mpsc;
use tower_http::cors::AllowOrigin;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};

/// instance 級功能開關：未啟用一律 404（回應與 fallback 一致，不暴露功能存在）。
/// 掛在 nest 點、包在 with_auth 外層 —— 關閉的功能連 401 都不回。
pub(super) fn with_feature(
    state: AppState,
    feature: Feature,
    router: Router<AppState>,
) -> Router<AppState> {
    router.layer(middleware::from_fn_with_state(
        state,
        move |State(state): State<AppState>, req: Request, next: Next| async move {
            if state.get_settings().feature_enabled(feature) {
                next.run(req).await
            } else {
                AppError::from(RequestError::NotFound).into_response()
            }
        },
    ))
}

/// 公開且**完全無個人化**的 GET 回應用的快取標頭。
///
/// 沒有這個標頭時，任何直接打 `api.kawa.homes` 的請求都會一路到 PG —— 前端 SSR 那條有
/// Next Data Cache（`frontend/api/blogs.ts` 的 `revalidate`）擋著，但那是走內網
/// `http://backend:3000`、根本不經 nginx，對「有人拿 origin IP 直接洪水打公開 API」
/// 這個情境一點幫助都沒有。搭配 `deploy/nginx/nginx.conf` 的 `api_cache`，重複的匿名
/// GET 會停在 nginx，不再消耗 PG 連線池（20 條）。
///
/// - `max-age=30`：瀏覽器端，短到使用者幾乎不會察覺
/// - `s-maxage=60`：共用快取（nginx / CF）—— 與 `frontend/api/blogs.ts` 的
///   `revalidate: 60` 對齊，是本站既有的新鮮度預期
///
/// ⚠️ **這一層不會被 `updateTag('blogs')` 失效**。後台存檔後，Next 那層立刻更新，但
/// nginx / 瀏覽器這層最久要等 60 秒。刻意選 60（而非 blog 詳情原本的 `revalidate: 300`）
/// 就是為了把這個新鮮度回退壓在一分鐘內。要立刻看到改動請 hard reload。
///
/// ⚠️ **只掛在沒有任何 per-user 差異的端點上**。留言列表刻意不掛：訪客送出留言後會馬上
/// 重讀那份列表（`frontend/api/blog-comments.ts` 是 `cache: "no-store"`），
/// 快取住等於「我剛留的言不見了」。
pub(super) fn public_cache() -> [(header::HeaderName, HeaderValue); 1] {
    [(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=30, s-maxage=60"),
    )]
}

pub(super) fn with_auth(state: AppState, router: Router<AppState>) -> Router<AppState> {
    // audit 掛在 auth 內層：auth 先跑塞入 AuthenticatedUser，audit 直接讀 extension，不重複 decode JWT
    router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::audit::audit_log,
        ))
        .layer(middleware::from_fn_with_state(
            state,
            crate::middleware::auth::authorize_and_load,
        ))
}

/// member 版的 `with_auth`：驗證會員身分 + 稽核（audit 掛內層，讀 `AuthenticatedMember`）。
///
/// `/member/*` 一直是直接掛 `authorize_member`、跳過 audit 的，於是「會員改了什麼、
/// 刪了什麼」零紀錄，出事只能靠 DB 現值猜。
///
/// **只有資料 CRUD 的四支走這裡**（portfolio / ledger / invoices / lotto）。
/// `vocab` 刻意不掛：它每答一題就是一個 `POST /runs/{id}/answer`，掛上去等於用 180 天
/// 保留期的稽核表存遊戲操作，而那些事件的稽核價值近乎零。
/// audit middleware 對 member 也只記非 GET（見 `middleware/audit.rs`）。
pub(super) fn with_member_auth(state: AppState, router: Router<AppState>) -> Router<AppState> {
    router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::audit::audit_log,
        ))
        .layer(middleware::from_fn_with_state(
            state,
            crate::middleware::auth::authorize_member,
        ))
}

pub async fn app(log_rx: mpsc::Receiver<LogEntry>) -> Router {
    let (state, audit_rx) = AppState::new().await;

    sqlx::migrate!("./migrations")
        .run(state.get_pool())
        .await
        .expect("migration failed");

    state.reload_settings().await;

    crate::services::oauth::OAuthProvider::warn_if_partially_configured(
        state.get_config(),
        &state.get_settings(),
    );

    let cors_origins: Vec<HeaderValue> = state
        .get_settings()
        .get("cors_allowed_origins")
        .unwrap_or_else(|| "https://kawa.homes".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // 兩個批次寫入器：WARN+ 的 log 與 /admin/* 的稽核紀錄。
    // 都刻意不在請求路徑上碰 DB —— 尖峰時不與真正的查詢搶那 20 條連線。
    tokio::spawn(crate::logging::log_writer(log_rx, state.get_pool().clone()));
    tokio::spawn(crate::services::audit_logs::audit_writer(
        audit_rx,
        state.get_pool().clone(),
    ));

    initialize_scheduler(state.clone()).await;

    // 重啟 resume：把 pending / downloading 的 torrent 補回 session（功能關閉時跳過）
    if state.get_settings().feature_enabled(Feature::Torrents) {
        tokio::spawn(crate::services::torrents::sync_active(state.clone()));
    }

    // 遊戲計時掃描：偵測行棋方時鐘耗盡卻無人走步 → 主動判負（每遊戲一個 watcher）
    for hub in state.games().all() {
        hub.spawn_watcher(state.clone());
    }

    let upload_path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string());

    // 複製成 bool 讓 span 的閉包不必抓整個 AppState（閉包要 Clone + Send + Sync）
    let trust_cf_header = state.get_config().trust_cf_header;

    Router::new()
        .nest("/admin", admin::new(state.clone()))
        .nest("/blogs", with_feature(state.clone(), Feature::Blog, blogs::new(state.clone())))
        .nest("/tools", with_feature(state.clone(), Feature::Tools, tools::new(state.clone())))
        .nest("/ws", ws::new(state.clone()))
        .nest("/roster", with_feature(state.clone(), Feature::Roster, roster::new(state.clone())))
        .nest("/members", members::new(state.clone()))
        .nest("/messages", with_feature(state.clone(), Feature::Message, messages::new(state.clone())))
        .nest("/member/portfolio", with_feature(state.clone(), Feature::Portfolio, portfolio::new(state.clone())))
        .nest("/member/ledger", with_feature(state.clone(), Feature::Ledger, ledger::new(state.clone())))
        .nest("/member/invoices", with_feature(state.clone(), Feature::Invoices, invoices::new(state.clone())))
        .nest("/member/lotto", with_feature(state.clone(), Feature::Lotto, lotto::new(state.clone())))
        .nest("/member/vocab", with_feature(state.clone(), Feature::Vocab, vocab::new(state.clone())))
        .nest("/oauth", oauth::new(state.clone()))
        .nest("/logs", logs::new(state.clone()))
        .nest("/metrics", metrics::new(state.clone()))
        .nest("/settings", app_settings::public())
        // 生產不會被打到（nginx 的 media vhost 直出磁碟，api vhost 的 /uploads/ 已 301 過去），
        // 但本地開發沒有 nginx，這是唯一能瀏覽自己剛上傳的圖的路徑 —— 本地把
        // app_settings.upload_base_url 設成 http://127.0.0.1:3000/uploads 就靠這行。
        .nest_service("/uploads", ServeDir::new(&upload_path))
        // 存活探針：不查 DB / Redis，純粹回報「行程活著且在收請求」，給外部 uptime 監控打
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "ok" })) }))
        // fallback 需在 layer 之前註冊，否則不會被下面的 request_id / TraceLayer 包住（404 也要有追蹤 id）
        // 形狀刻意與 AppError 一致（含 request_id），全站錯誤只有一種格式；with_feature 的 404 同源
        .fallback(|| async { AppError::from(RequestError::NotFound) })
        .layer(DefaultBodyLimit::disable())
        // 必須與 deploy/nginx/nginx.conf 的 client_max_body_size 10m 相等。
        // 不相等的話，落在兩者之間的檔案會由 nginx 放行、後端才 413，而 api vhost
        // 開了 proxy_request_buffering off，client 端看到的是上傳到一半斷掉。
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_origin(AllowOrigin::list(cors_origins))
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                // 讓瀏覽器端 JS 可讀到追蹤 id，方便回報問題時附上
                .expose_headers([header::HeaderName::from_static("x-request-id")]),
        )
        // handler 上限：逾時放棄產生回應，回 408（body 由下面的 error_shape 正規化）。
        //
        // 沒有這層時，一個卡住的 handler（上游不回、DB 慢查詢、鎖等待）會一直佔著
        // tokio worker 與那條 PG 連線；1 核 1G 上幾個就吃光。
        //
        // **60 秒是對齊 nginx 而不是隨手挑的**：`deploy/nginx/conf.d/api.kawa.homes.conf`
        // 的 `location /` 吃預設 `proxy_read_timeout 60s`，也就是說超過 60 秒的請求
        // client 早就收到 504 了，backend 再跑下去純粹是浪費。
        //
        // ⚠️ **不要調得更短**：api vhost 開了 `proxy_request_buffering off`，圖片上傳的
        // client body 是邊傳邊進 handler 的，計時包含使用者的上傳時間 —— 10MB 走慢速
        // 行動網路可以超過半分鐘，短逾時會把正常上傳打成 408。
        //
        // 對長連線無影響：計時只到「回應產生」為止，WS 的 101 與 torrent 下載的
        // `ServeFile` 都是先回 header 再串流，串流時間不算在內。
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(60),
        ))
        // 把 layer / Router 直接吐的錯誤（408、413、405）換成統一形狀 + 落 log。
        // 必須掛在 `RequestBodyLimitLayer` 與 Router **外層**（= 程式碼順序在其後）才看得到
        // 它們的回應，同時在 request_id middleware 內層，才拿得到 task-local 的追蹤 id。
        .layer(middleware::from_fn(
            crate::middleware::error_shape::error_shape,
        ))
        // handler panic → 500（帶 request_id）+ 一筆 ERROR。掛在 TraceLayer **內層**：
        // panic 被接住的當下 request span 仍然活著、request_id 的 task-local 也還在
        // scope 內，那筆 ERROR 才對得回這次請求。往外掛就兩個都拿不到。
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(
            crate::errors::handle_panic,
        ))
        // 每請求一條 span：method / path / query / ip / request_id（由下方 middleware 塞入 extensions）
        //
        // `on_response` 補一行 INFO：沒有它，`logs` 表只有出錯時才有紀錄，
        // `GET /logs/request/{id}` 對一個成功請求回空陣列 —— 答不出「這個 request 回什麼
        // status、跑多久」，追「變慢」或「回 200 但資料錯」完全沒有時間軸。
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(move |req: &axum::http::Request<_>| {
                    let request_id = req
                        .extensions()
                        .get::<crate::middleware::request_id::RequestId>()
                        .map(|r| r.0.as_str())
                        .unwrap_or("-");
                    // query 要遮罩：/ws?ticket= 與 oauth callback 的 ?code= 都是憑證
                    let query = req
                        .uri()
                        .query()
                        .map(crate::utils::redact::redact_query)
                        .unwrap_or_default();
                    // 與限流 / 到訪統計同一套判斷（`utils::net`），漂移會造成
                    // 「限流認得出真 IP、log 認不出」這種最難查的不一致
                    let socket_ip = req
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                        .map(|ci| ci.0.ip());
                    let ip = crate::utils::net::client_ip(trust_cf_header, req.headers(), socket_ip);
                    tracing::info_span!(
                        "request",
                        method = %req.method(),
                        path = %req.uri().path(),
                        query = %query,
                        ip = %ip,
                        request_id = %request_id,
                    )
                })
                .on_response(
                    |res: &axum::http::Response<_>, latency: Duration, _: &tracing::Span| {
                        // 專屬 target `<crate>::access`：量級跟其他 INFO 差一個數量級，
                        // 要能單獨關掉（`RUST_LOG=...,api_server_axum::access=off`）而不影響
                        // 其餘。長在 crate 名底下才會被預設 filter 的前綴比對自動涵蓋。
                        tracing::info!(
                            target: crate::logging::ACCESS_TARGET,
                            status = res.status().as_u16(),
                            latency_ms = latency.as_millis() as u64,
                            "request completed"
                        );
                    },
                ),
        )
        // 最外層：產生 request_id → 供上面 span 讀取、寫回 response header、供錯誤 body 回溯
        .layer(middleware::from_fn(
            crate::middleware::request_id::request_id,
        ))
        .with_state(state)
}

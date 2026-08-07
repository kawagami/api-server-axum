mod games;
mod errors;
mod jobs;
mod logging;
mod middleware;
mod repositories;
mod routes;
mod scheduler;
mod services;
mod state;
mod storage;
mod structs;
mod utils;

use std::{env::var, net::SocketAddr};
use tokio::{net::TcpListener, signal, sync::mpsc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // ⚠ 必須在 tracing 初始化之前：`EnvFilter::try_from_default_env()` 讀的是真正的
    // process env，dotenv 晚一步的話 `backend/.env` 裡的 RUST_LOG 會靜默失效
    // （2026-08-08 之前就是這樣，那個逃生門只有 `export RUST_LOG=` 走得通）。
    dotenvy::dotenv().ok();

    let (log_tx, log_rx) = mpsc::channel::<logging::LogEntry>(1000);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                default_log_filter().into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true),
        )
        .with(logging::DbLogLayer::new(log_tx))
        .init();

    let app = routes::app(log_rx).await;

    // 設定伺服器監聽的主機與埠號
    let host = var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()); // 預設監聽所有 IP
    // ⚠ APP_PORT 只服務「本機直跑」的情境（3000 被佔時換一個）。生產的 3000 被
    // deploy/nginx/conf.d/02-proxy.conf 的 upstream 與 kawa.env 的 API_URL 寫死，
    // 只改這個 env 會讓 nginx 照打 3000 → 502，故生產的 kawa.env 刻意不放這個 key。
    let port = var("APP_PORT").unwrap_or_else(|_| "3000".to_string()); // 預設使用 3000 埠
    let bind_addr = format!("{}:{}", host, port); // 組合完整的監聽地址
    let listener = TcpListener::bind(&bind_addr).await.unwrap(); // 綁定 TCP 監聽埠
    // 啟動事件是 info 不是 debug —— 生產跑在 info，這行是 stdout 上「行程有沒有起來」的唯一依據
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    // 啟動 Axum 伺服器，並加入優雅關閉（graceful shutdown）機制
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    tracing::info!("server stopped, flushing logs");
    // `log_writer` 的 buffer 最多攢 500ms 才落地，而 DbLogLayer 的 sender 活在全域
    // subscriber 裡永遠不會被 drop（收不到 channel 關閉），沒辦法 join 它。
    // 等一輪 flush interval 是最小代價的做法：不等的話每次部署都會吃掉關機前那半秒的 log。
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
}

/// 未設 `RUST_LOG` 時的預設 filter。
///
/// 生產（release image，含所有商家 instance）跑 `info` + `tower_http=warn`：
/// - debug 會讓 tower_http 對**每個請求**印 started/finished 兩行，而 compose 給
///   backend 的 rotation 是「稀疏」規格（10m×3）
/// - `tower_http=warn` 只掐掉那兩行，5xx 的 `on_failure` 是 ERROR，照樣留著
///   （CLAUDE.md 講的「一個 5xx 落 3 筆」不受影響）
/// - `logs` 表的落地門檻另有 app_settings 的 `log_db_level`，但**只能在這個天花板
///   底下調** —— EnvFilter 掛在 registry 上是全域 filter，被它擋掉的 event 到不了
///   `DbLogLayer`。所以這裡是 info，那邊的上限才會是 INFO
///
/// 本機 `cargo run`（debug build）維持全 debug，不必設任何 env。
fn default_log_filter() -> String {
    let crate_name = env!("CARGO_CRATE_NAME");
    if cfg!(debug_assertions) {
        format!("{crate_name}=debug,tower_http=debug")
    } else {
        format!("{crate_name}=info,tower_http=warn")
    }
}

// 監聽系統訊號，實作優雅關閉機制
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler"); // 捕捉 Ctrl+C 訊號
    };

    // Unix 系統專屬：捕捉 SIGTERM 訊號
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // 非 Unix 系統：無限等待（模擬不支援 SIGTERM）
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // 讓 terminate 變數永遠不會完成

    // 監聽兩種訊號，若其中一個發生則進行關閉。
    // 兩邊都記一行：沒有這個的話「被 SIGTERM 正常收掉」與「被 OOM killer 砍掉」
    // 在 log 上長得一模一樣（2026-07-18 那次 OOM 就是靠 dmesg 才分辨出來的）。
    tokio::select! {
        _ = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}

mod errors;
mod image_processor;
mod jobs;
mod repositories;
mod routes;
mod scheduler;
mod services;
mod state;
mod structs;
mod utils;

use std::env::var;
use tokio::{net::TcpListener, signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 初始化日誌系統，根據環境變數設定日誌層級，預設為 "template_axum=debug"
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "template_axum=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true) // 顯示檔案名稱
                .with_line_number(true), // 顯示行號
        ) // 設定格式化日誌輸出
        .init();

    dotenvy::dotenv().ok(); // 載入 .env 環境變數檔案（若存在）

    let app = routes::app().await; // 初始化 Axum 應用程式

    // 設定伺服器監聽的主機與埠號
    let host = var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()); // 預設監聽所有 IP
    let port = var("APP_PORT").unwrap_or_else(|_| "3000".to_string()); // 預設使用 3000 埠
    let bind_addr = format!("{}:{}", host, port); // 組合完整的監聽地址
    let listener = TcpListener::bind(&bind_addr).await.unwrap(); // 綁定 TCP 監聽埠
    tracing::debug!("listening on {}", listener.local_addr().unwrap()); // 記錄監聽的地址

    // 啟動 Axum 伺服器，並加入優雅關閉（graceful shutdown）機制
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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

    // 監聽兩種訊號，若其中一個發生則進行關閉
    tokio::select! {
        _ = ctrl_c => {}, // 若偵測到 Ctrl+C，則繼續執行關閉程序
        _ = terminate => {}, // 若偵測到 SIGTERM，則繼續執行關閉程序
    }
}

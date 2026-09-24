//! WebSocket（`/ws`）的連線層邏輯。route（`routes/ws.rs`）只留握手與 HTTP 端點的形狀，
//! 其餘都在這裡（2026-09-24 自 `routes/ws.rs` 搬來 socket 生命週期，函式內容未動）：
//! - `ticket`：admin 身分的一次性連線票
//! - `guard`：連線防護 —— per-IP 連線上限、每連線收訊令牌桶
//! - `socket`：單條連線的生命週期（收訊迴圈、ping/pong、遊戲分派、斷線清理）
//! - `connections`：後台的連線清單與點對點送訊
//!
//! 送出順序、ping/pong、連線防護的不變式見 ARCHITECTURE.md「WebSocket 推送」。

mod connections;
mod guard;
mod socket;
mod ticket;

pub use connections::*;
pub use guard::*;
pub use socket::*;
pub use ticket::*;

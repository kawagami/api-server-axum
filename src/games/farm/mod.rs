//! 農場經營（家庭版 worker-placement）。N 人、完全資訊、worker-placement。
//! 機制原創、零桌遊素材。`engine` 純邏輯（可單測）；`hub` + `service` 為 WS 串接（同阿瓦隆模式）。

/// 遊戲代號（registry key / WS 信封 `game` 欄）的單一來源。
pub const NAME: &str = "farm";

pub mod engine;
pub mod hub;
pub mod service;

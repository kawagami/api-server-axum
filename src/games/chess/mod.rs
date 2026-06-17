//! 線上象棋：純函式規則引擎（`engine` + `types`）與配對/房間狀態（`hub`）。
//! WS 串接邏輯在 `crate::services::chess`。

pub mod engine;
pub mod hub;
pub mod types;

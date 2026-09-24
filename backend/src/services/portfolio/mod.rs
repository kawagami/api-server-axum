//! member 持股損益。依關注點拆成三塊（2026-09-24 自單檔拆出，函式內容未動）：
//! - `crud`：持股增刪改查
//! - `pricing`：summary / history 的三層快取管線（Redis → DB → TWSE）與上游抓取預算 `UpstreamBudget`
//! - `math`：純計算 —— 除權息還原因子、最新損益、期間增減、逐日還原成本 + 單元測試
//!
//! 預算、併發上限與除權息的坑見 ARCHITECTURE.md「持股損益」。

mod crud;
mod pricing;
mod math;

pub use crud::*;
pub use pricing::*;

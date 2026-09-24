//! 單字闖關的資料存取，依關注點拆成四塊（2026-09-24 自單檔拆出，函式內容未動）：
//! - `words`：出題抽字（依 id / 隨機 / 難度上下界 / 干擾項）
//! - `stats`：會員學習進度（錯題本、逐題統計、複習池、學習天數）
//! - `runs`：對局落地、個人最佳、經驗值、週期排行榜
//! - `admin`：後台題庫管理
//!
//! ⚠️ **多語言鐵律**：所有「依條件篩選」words / vocab_runs 的查詢都必須帶 language，
//! 見 ARCHITECTURE.md「單字闖關」。

mod words;
mod stats;
mod runs;
mod admin;

pub use words::*;
pub use stats::*;
pub use runs::*;
pub use admin::*;

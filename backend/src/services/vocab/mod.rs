//! 單字闖關（member 生存模式）的業務邏輯，依關注點拆成六塊（2026-09-24 自單檔拆出，函式內容未動）：
//! - `engine`：純函式（等級曲線、經驗值、難度窗口、題型、例句挖空、限時、週期起點、連續天數）+ 全部單元測試
//! - `question`：出題（抽字、干擾項、複習池）
//! - `run`：對局生命週期（開局、Redis 存取、結算、提早結束）
//! - `answer`：答題判定與續局
//! - `stats`：錯題本、週期排行榜、個人統計
//! - `admin`：後台題庫管理
//!
//! 進行中對局存 Redis、正解只在 server 端；見 ARCHITECTURE.md「單字闖關」。

mod engine;
mod question;
mod run;
mod answer;
mod stats;
mod admin;

pub use run::*;
pub use answer::*;
pub use stats::*;
pub use admin::*;

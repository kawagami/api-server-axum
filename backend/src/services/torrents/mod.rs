//! Torrent 下載：磁力連結 → librqbit 下載 → 簽名連結取檔。依關注點拆成六塊（2026-09-24 自單檔拆出，函式內容未動）：
//! - `manager`：設定預設值、`TorrentManager`（librqbit session + 併發名額 `active` map）
//! - `tasks`：對外 CRUD（新增 / 列表 / 詳情 / 重跑 / 刪除）、擁有者檢查、排程清理
//! - `lifecycle`：補位 `sync_active`、啟動（metadata 讓位）、進度 watcher、完成 / 失敗收尾
//! - `session`：從 librqbit session 移除任務的 helper
//! - `download_links`：短效簽名下載連結的發放與解析
//! - `storage`：磁碟空間與配額統計
//!
//! 狀態流與不變式見 ARCHITECTURE.md「Torrent 下載」。

mod manager;
mod tasks;
mod lifecycle;
mod session;
mod download_links;
mod storage;

pub use manager::*;
pub use tasks::*;
pub use lifecycle::*;
pub use download_links::*;
pub use storage::*;

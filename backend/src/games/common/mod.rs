//! 通用對戰框架。
//! 回合制 2 人：`GameEngine` trait + 泛型大廳/桌位/配對/計時/斷線（`engine`/`hub`/`service`），
//! 各遊戲只需 impl `GameEngine`（見 `games::chess::game` 等）。
//! N 人房（avalon/farm）：`RoomKind` trait + 泛型大廳/房間/斷線（`room`）。

pub mod engine;
pub mod hub;
pub mod room;
pub mod service;

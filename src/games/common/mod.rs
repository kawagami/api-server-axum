//! 通用回合制 2 人對戰框架：`GameEngine` trait + 泛型大廳/桌位/配對/計時/斷線。
//! 各遊戲只需 impl `GameEngine`（見 `games::chess::game` 等）。

pub mod engine;
pub mod hub;
pub mod service;

//! 對戰遊戲：共用框架 `common`（GameEngine trait + 大廳/桌位/配對/計時），
//! 各遊戲純引擎 + adapter 放各子模組。WS 分派在 `crate::routes::ws`。

pub mod avalon;
pub mod banqi;
pub mod chess;
pub mod common;
pub mod go;
pub mod gomoku;
pub mod registry;
pub mod western_chess;

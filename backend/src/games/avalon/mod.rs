//! 阿瓦隆：N 人社交推理。獨立子系統（不走 GameEngine 2 人框架）。
//! `roles` + `engine` 為純邏輯；`hub` + `service` 為 WS 串接（私有角色推送 + 階段機 + chat）。

/// 遊戲代號（registry key / WS 信封 `game` 欄）的單一來源。
pub const NAME: &str = "avalon";

pub mod engine;
pub mod hub;
pub mod roles;
pub mod service;

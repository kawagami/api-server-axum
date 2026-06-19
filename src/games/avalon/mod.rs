//! 阿瓦隆：N 人社交推理。獨立子系統（不走 GameEngine 2 人框架）。
//! `roles` + `engine` 為純邏輯；`hub` + `service` 為 WS 串接（私有角色推送 + 階段機 + chat）。

pub mod engine;
pub mod hub;
pub mod roles;
pub mod service;

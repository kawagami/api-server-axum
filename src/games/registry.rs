//! 遊戲註冊表：用 enum 收斂所有遊戲 hub，避免 AppState / ws / routes 每加一款遊戲就改一次。
//!
//! 新增遊戲只需動本檔：① `AnyHub` 加 variant ② 三個 method 各加一臂 ③ `new()` 註冊一行。
//! state.rs / ws.rs / routes.rs 全 game-agnostic，不必再改。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::Mutex;

use crate::games::banqi::game::BanqiGame;
use crate::games::chess::game::ChessGame;
use crate::games::common::engine::GameEngine;
use crate::games::common::hub::{GameHub, HubInner};
use crate::games::common::service;
use crate::games::gomoku::game::GomokuGame;
use crate::state::AppState;

fn new_hub<E: GameEngine>() -> GameHub<E> {
    Arc::new(Mutex::new(HubInner::default()))
}

/// 型別化的遊戲 hub。每臂為具體 `E`，match 後單型化呼叫泛型 `service::*`（無 dyn / 無 async-trait）。
pub enum AnyHub {
    Chess(GameHub<ChessGame>),
    Gomoku(GameHub<GomokuGame>),
    Banqi(GameHub<BanqiGame>),
}

impl AnyHub {
    pub async fn handle(&self, state: &AppState, who: SocketAddr, value: &Value) -> bool {
        match self {
            AnyHub::Chess(h) => service::handle(h, state, who, value).await,
            AnyHub::Gomoku(h) => service::handle(h, state, who, value).await,
            AnyHub::Banqi(h) => service::handle(h, state, who, value).await,
        }
    }

    pub async fn disconnect(&self, state: &AppState, who: SocketAddr) {
        match self {
            AnyHub::Chess(h) => service::handle_disconnect(h, state, who).await,
            AnyHub::Gomoku(h) => service::handle_disconnect(h, state, who).await,
            AnyHub::Banqi(h) => service::handle_disconnect(h, state, who).await,
        }
    }

    pub fn spawn_watcher(&self, state: AppState) {
        match self {
            AnyHub::Chess(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
            AnyHub::Gomoku(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
            AnyHub::Banqi(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
        };
    }
}

/// 所有遊戲 hub，依 `game` 名索引。
pub struct GameRegistry(HashMap<&'static str, AnyHub>);

impl GameRegistry {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(ChessGame::NAME, AnyHub::Chess(new_hub()));
        m.insert(GomokuGame::NAME, AnyHub::Gomoku(new_hub()));
        m.insert(BanqiGame::NAME, AnyHub::Banqi(new_hub()));
        GameRegistry(m)
    }

    pub fn get(&self, game: &str) -> Option<&AnyHub> {
        self.0.get(game)
    }

    pub fn all(&self) -> impl Iterator<Item = &AnyHub> {
        self.0.values()
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

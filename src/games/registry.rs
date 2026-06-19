//! 遊戲註冊表：用 enum 收斂所有遊戲 hub，避免 AppState / ws / routes 每加一款遊戲就改一次。
//!
//! 新增遊戲只需動本檔：① `AnyHub` 加 variant ② 三個 method 各加一臂 ③ `new()` 註冊一行。
//! state.rs / ws.rs / routes.rs 全 game-agnostic，不必再改。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::Mutex;

use crate::games::avalon::hub::{AvalonHub, AvalonHubInner};
use crate::games::avalon::service as avalon_service;
use crate::games::banqi::game::BanqiGame;
use crate::games::chess::game::ChessGame;
use crate::games::common::engine::GameEngine;
use crate::games::common::hub::{GameHub, HubInner};
use crate::games::common::service;
use crate::games::farm::hub::{FarmHub, FarmHubInner};
use crate::games::farm::service as farm_service;
use crate::games::go::game::GoGame;
use crate::games::gomoku::game::GomokuGame;
use crate::games::western_chess::game::WesternChessGame;
use crate::state::AppState;

fn new_hub<E: GameEngine>() -> GameHub<E> {
    Arc::new(Mutex::new(HubInner::default()))
}

/// 型別化的遊戲 hub。每臂為具體 `E`，match 後單型化呼叫泛型 `service::*`（無 dyn / 無 async-trait）。
pub enum AnyHub {
    Chess(GameHub<ChessGame>),
    Gomoku(GameHub<GomokuGame>),
    Banqi(GameHub<BanqiGame>),
    WesternChess(GameHub<WesternChessGame>),
    Go(GameHub<GoGame>),
    /// 阿瓦隆：獨立子系統（非 GameEngine 2 人框架）。
    Avalon(AvalonHub),
    /// 農場經營：N 人 worker-placement，獨立子系統。
    Farm(FarmHub),
}

impl AnyHub {
    pub async fn handle(&self, state: &AppState, who: SocketAddr, value: &Value) -> bool {
        match self {
            AnyHub::Chess(h) => service::handle(h, state, who, value).await,
            AnyHub::Gomoku(h) => service::handle(h, state, who, value).await,
            AnyHub::Banqi(h) => service::handle(h, state, who, value).await,
            AnyHub::WesternChess(h) => service::handle(h, state, who, value).await,
            AnyHub::Go(h) => service::handle(h, state, who, value).await,
            AnyHub::Avalon(h) => avalon_service::handle(h, state, who, value).await,
            AnyHub::Farm(h) => farm_service::handle(h, state, who, value).await,
        }
    }

    pub async fn disconnect(&self, state: &AppState, who: SocketAddr) {
        match self {
            AnyHub::Chess(h) => service::handle_disconnect(h, state, who).await,
            AnyHub::Gomoku(h) => service::handle_disconnect(h, state, who).await,
            AnyHub::Banqi(h) => service::handle_disconnect(h, state, who).await,
            AnyHub::WesternChess(h) => service::handle_disconnect(h, state, who).await,
            AnyHub::Go(h) => service::handle_disconnect(h, state, who).await,
            AnyHub::Avalon(h) => avalon_service::handle_disconnect(h, state, who).await,
            AnyHub::Farm(h) => farm_service::handle_disconnect(h, state, who).await,
        }
    }

    pub fn spawn_watcher(&self, state: AppState) {
        match self {
            AnyHub::Chess(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
            AnyHub::Gomoku(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
            AnyHub::Banqi(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
            AnyHub::WesternChess(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
            AnyHub::Go(h) => tokio::spawn(service::timeout_watcher(h.clone(), state)),
            AnyHub::Avalon(_) | AnyHub::Farm(_) => return, // 無計時，不需 watcher
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
        m.insert(WesternChessGame::NAME, AnyHub::WesternChess(new_hub()));
        m.insert(GoGame::NAME, AnyHub::Go(new_hub()));
        m.insert("avalon", AnyHub::Avalon(Arc::new(Mutex::new(AvalonHubInner::default()))));
        m.insert("farm", AnyHub::Farm(Arc::new(Mutex::new(FarmHubInner::default()))));
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

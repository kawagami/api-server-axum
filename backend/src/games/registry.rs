//! 遊戲註冊表：用 enum 收斂所有遊戲 hub，避免 AppState / ws / routes 每加一款遊戲就改一次。
//!
//! 新增遊戲只需動本檔：① `AnyHub` 加 variant ② 三個 method 各加一臂 ③ `new()` 註冊一行。
//! state.rs / ws.rs / routes.rs 全 game-agnostic，不必再改。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::games::avalon::hub::{AvalonHub, AvalonHubInner, RoomState as AvalonRoomState};
use crate::games::avalon::service as avalon_service;
use crate::games::banqi::game::BanqiGame;
use crate::games::chess::game::ChessGame;
use crate::games::common::engine::GameEngine;
use crate::games::common::hub::{GameHub, HubInner, TableState};
use crate::games::common::service;
use crate::games::farm::hub::{FarmHub, FarmHubInner, RoomState as FarmRoomState};
use crate::games::farm::service as farm_service;
use crate::games::go::game::GoGame;
use crate::games::gomoku::game::GomokuGame;
use crate::games::western_chess::game::WesternChessGame;
use crate::state::AppState;

fn new_hub<E: GameEngine>() -> GameHub<E> {
    Arc::new(Mutex::new(HubInner::default()))
}

/// 單一遊戲的即時對局統計（唯讀快照，給 admin 後台用）。匿名對戰，不含玩家身份。
#[derive(Serialize)]
pub struct GameSummary {
    /// 遊戲代號（registry key）。
    pub game: &'static str,
    /// 等待中的桌 / 房數（尚未開局）。
    pub waiting: usize,
    /// 進行中的桌 / 房數（對局中）。
    pub playing: usize,
    /// 進行中對局的總人數。
    pub players_in_game: usize,
    /// 快速配對佇列人數（無此機制的子系統為 0）。
    pub queued: usize,
    /// 訂閱大廳更新的連線數。
    pub lobby: usize,
}

/// 2 人泛型 hub 的統計。
async fn summarize_2p<E>(game: &'static str, hub: &GameHub<E>) -> GameSummary {
    let h = hub.lock().await;
    let mut waiting = 0;
    let mut playing = 0;
    for t in h.tables.values() {
        match t.state {
            TableState::Waiting { .. } => waiting += 1,
            TableState::Playing(_) => playing += 1,
        }
    }
    GameSummary {
        game,
        waiting,
        playing,
        players_in_game: playing * 2,
        queued: h.queue.len(),
        lobby: h.lobby.len(),
    }
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

    /// 唯讀統計快照。各臂鎖各自 mutex，回傳大廳層級的計數（不含玩家身份 / 盤面）。
    pub async fn snapshot(&self, game: &'static str) -> GameSummary {
        match self {
            AnyHub::Chess(h) => summarize_2p(game, h).await,
            AnyHub::Gomoku(h) => summarize_2p(game, h).await,
            AnyHub::Banqi(h) => summarize_2p(game, h).await,
            AnyHub::WesternChess(h) => summarize_2p(game, h).await,
            AnyHub::Go(h) => summarize_2p(game, h).await,
            AnyHub::Avalon(h) => {
                let h = h.lock().await;
                let mut waiting = 0;
                let mut playing = 0;
                let mut players_in_game = 0;
                for r in h.rooms.values() {
                    match r.state {
                        AvalonRoomState::Waiting => waiting += 1,
                        AvalonRoomState::Playing(_) => {
                            playing += 1;
                            players_in_game += r.players.len();
                        }
                    }
                }
                GameSummary {
                    game,
                    waiting,
                    playing,
                    players_in_game,
                    queued: 0,
                    lobby: h.lobby.len(),
                }
            }
            AnyHub::Farm(h) => {
                let h = h.lock().await;
                let mut waiting = 0;
                let mut playing = 0;
                let mut players_in_game = 0;
                for r in h.rooms.values() {
                    match r.state {
                        FarmRoomState::Waiting => waiting += 1,
                        FarmRoomState::Playing(_) => {
                            playing += 1;
                            players_in_game += r.players.len();
                        }
                    }
                }
                GameSummary {
                    game,
                    waiting,
                    playing,
                    players_in_game,
                    queued: 0,
                    lobby: h.lobby.len(),
                }
            }
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
        m.insert(crate::games::avalon::NAME, AnyHub::Avalon(Arc::new(Mutex::new(AvalonHubInner::default()))));
        m.insert(crate::games::farm::NAME, AnyHub::Farm(Arc::new(Mutex::new(FarmHubInner::default()))));
        GameRegistry(m)
    }

    pub fn get(&self, game: &str) -> Option<&AnyHub> {
        self.0.get(game)
    }

    pub fn all(&self) -> impl Iterator<Item = &AnyHub> {
        self.0.values()
    }

    /// 所有遊戲的即時統計快照，依 game 名排序（輸出穩定）。
    pub async fn summaries(&self) -> Vec<GameSummary> {
        let mut out = Vec::with_capacity(self.0.len());
        for (&name, hub) in &self.0 {
            out.push(hub.snapshot(name).await);
        }
        out.sort_by_key(|s| s.game);
        out
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

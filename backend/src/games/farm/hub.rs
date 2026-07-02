//! 農場經營大廳 / 房間記憶體狀態（純資料，無 WS 依賴）。2–4 人，重啟即丟失。

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::engine::GameState;

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 4;

pub type FarmHub = Arc<Mutex<FarmHubInner>>;

#[derive(Default)]
pub struct FarmHubInner {
    pub rooms: HashMap<u64, Room>,
    pub conn_room: HashMap<SocketAddr, u64>,
    pub lobby: HashSet<SocketAddr>,
    pub next_id: u64,
}

impl FarmHubInner {
    pub fn is_committed(&self, who: SocketAddr) -> bool {
        self.conn_room.contains_key(&who)
    }
}

pub struct Room {
    pub id: u64,
    pub name: String,
    /// 座位順序＝加入順序；對局開始後固定。index = 玩家。
    pub players: Vec<SocketAddr>,
    pub names: Vec<String>,
    pub host: SocketAddr,
    pub state: RoomState,
}

// 房間數少、Waiting 短命，Playing 大 payload 不 Box 也無記憶體壓力
#[allow(clippy::large_enum_variant)]
pub enum RoomState {
    Waiting,
    Playing(GameState),
}

impl Room {
    pub fn seat_of(&self, who: SocketAddr) -> Option<usize> {
        self.players.iter().position(|&p| p == who)
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= MAX_PLAYERS
    }

    pub fn can_start(&self) -> bool {
        matches!(self.state, RoomState::Waiting)
            && (MIN_PLAYERS..=MAX_PLAYERS).contains(&self.players.len())
    }
}

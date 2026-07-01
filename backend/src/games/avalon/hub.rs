//! 阿瓦隆大廳 / 房間記憶體狀態（純資料，無 WS 依賴）。N 人房，重啟即丟失。

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::engine::AvalonState;
use super::roles::Options;

pub const MIN_PLAYERS: usize = 5;
pub const MAX_PLAYERS: usize = 10;

pub type AvalonHub = Arc<Mutex<AvalonHubInner>>;

#[derive(Default)]
pub struct AvalonHubInner {
    pub rooms: HashMap<u64, Room>,
    pub conn_room: HashMap<SocketAddr, u64>,
    pub lobby: HashSet<SocketAddr>,
    pub next_id: u64,
}

impl AvalonHubInner {
    pub fn is_committed(&self, who: SocketAddr) -> bool {
        self.conn_room.contains_key(&who)
    }
}

pub struct Room {
    pub id: u64,
    pub name: String,
    /// 座位順序＝加入順序；對局開始後固定。index = seat。
    pub players: Vec<SocketAddr>,
    pub names: Vec<String>,
    pub host: SocketAddr,
    pub options: Options,
    pub state: RoomState,
}

pub enum RoomState {
    Waiting,
    Playing(AvalonState),
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

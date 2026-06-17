//! 房間 / 配對的記憶體狀態（純資料，無 WS 依賴）。重啟即丟失。

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

use super::types::{GameState, Side};

pub const INITIAL_CLOCK_MS: i64 = 300_000; // 5:00
pub const INCREMENT_MS: i64 = 30_000; // Fischer +30s

pub type ChessHub = Arc<Mutex<ChessHubInner>>;

#[derive(Default)]
pub struct ChessHubInner {
    pub queue: VecDeque<SocketAddr>,
    pub rooms: HashMap<u64, Room>,
    /// 連線 → 所屬 room，快速反查。
    pub conn_room: HashMap<SocketAddr, u64>,
    pub next_id: u64,
}

pub struct Room {
    pub id: u64,
    pub red: SocketAddr,
    pub black: SocketAddr,
    pub state: GameState,
    pub red_ms: i64,
    pub black_ms: i64,
    /// 當前行棋方本回合開始時刻（server 單調時鐘）。
    pub turn_started_at: Instant,
    pub ended: bool,
}

impl Room {
    pub fn side_of(&self, who: SocketAddr) -> Option<Side> {
        if who == self.red {
            Some(Side::Red)
        } else if who == self.black {
            Some(Side::Black)
        } else {
            None
        }
    }

    pub fn clock_of(&self, side: Side) -> i64 {
        match side {
            Side::Red => self.red_ms,
            Side::Black => self.black_ms,
        }
    }
}

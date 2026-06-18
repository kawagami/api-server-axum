//! 大廳 / 桌位 / 對局的記憶體狀態（純資料，無 WS 依賴）。重啟即丟失。

use std::collections::{HashMap, HashSet, VecDeque};
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
    /// 快速配對佇列。
    pub queue: VecDeque<SocketAddr>,
    /// 所有桌（等待中 + 對戰中）。
    pub tables: HashMap<u64, Table>,
    /// 連線 → 所屬桌（host 等待中或對戰中皆含），快速反查。
    pub conn_table: HashMap<SocketAddr, u64>,
    /// 訂閱大廳更新的連線（在大廳頁、未入局）。
    pub lobby: HashSet<SocketAddr>,
    pub next_id: u64,
}

impl ChessHubInner {
    /// 連線是否已有承諾（在佇列或在桌）。一條連線同時只能在其一。
    pub fn is_committed(&self, who: SocketAddr) -> bool {
        self.conn_table.contains_key(&who) || self.queue.contains(&who)
    }
}

pub struct Table {
    pub id: u64,
    pub name: String,
    pub state: TableState,
}

pub enum TableState {
    /// 等待對手；`host` 已就座。
    Waiting { host: SocketAddr },
    /// 對戰中。
    Playing(Game),
}

/// 進行中的一局。
pub struct Game {
    pub red: SocketAddr,
    pub black: SocketAddr,
    pub state: GameState,
    pub red_ms: i64,
    pub black_ms: i64,
    /// 當前行棋方本回合開始時刻（server 單調時鐘）。
    pub turn_started_at: Instant,
    pub ended: bool,
}

impl Game {
    pub fn new(red: SocketAddr, black: SocketAddr, state: GameState) -> Self {
        Game {
            red,
            black,
            state,
            red_ms: INITIAL_CLOCK_MS,
            black_ms: INITIAL_CLOCK_MS,
            turn_started_at: Instant::now(),
            ended: false,
        }
    }

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

//! 泛型大廳 / 桌位 / 對局記憶體狀態（純資料，無 WS 依賴）。重啟即丟失。

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

use super::engine::{GameEngine, Side};

pub type GameHub<E> = Arc<Mutex<HubInner<E>>>;

pub struct HubInner<E> {
    /// 快速配對佇列。
    pub queue: VecDeque<SocketAddr>,
    /// 所有桌（等待中 + 對戰中）。
    pub tables: HashMap<u64, Table<E>>,
    /// 連線 → 所屬桌，快速反查。
    pub conn_table: HashMap<SocketAddr, u64>,
    /// 訂閱大廳更新的連線。
    pub lobby: HashSet<SocketAddr>,
    pub next_id: u64,
}

// 手動 impl Default（derive 會多加 `E: Default` 約束，實際不需要）。
impl<E> Default for HubInner<E> {
    fn default() -> Self {
        HubInner {
            queue: VecDeque::new(),
            tables: HashMap::new(),
            conn_table: HashMap::new(),
            lobby: HashSet::new(),
            next_id: 0,
        }
    }
}

impl<E> HubInner<E> {
    /// 連線是否已有承諾（在佇列或在桌）。一條連線同時只能在其一。
    pub fn is_committed(&self, who: SocketAddr) -> bool {
        self.conn_table.contains_key(&who) || self.queue.contains(&who)
    }
}

pub struct Table<E> {
    pub id: u64,
    pub name: String,
    pub state: TableState<E>,
}

pub enum TableState<E> {
    Waiting { host: SocketAddr },
    Playing(Game<E>),
}

/// 進行中的一局。`seats[0]` = First，`seats[1]` = Second。
pub struct Game<E> {
    pub seats: [SocketAddr; 2],
    pub engine: E,
    pub clock_ms: [i64; 2],
    /// 當前行棋方本回合開始時刻（server 單調時鐘）。
    pub turn_started_at: Instant,
    pub ended: bool,
}

impl<E: GameEngine> Game<E> {
    pub fn new(first: SocketAddr, second: SocketAddr) -> Self {
        Game {
            seats: [first, second],
            engine: E::initial(),
            clock_ms: [E::INITIAL_CLOCK_MS, E::INITIAL_CLOCK_MS],
            turn_started_at: Instant::now(),
            ended: false,
        }
    }

    pub fn side_of(&self, who: SocketAddr) -> Option<Side> {
        if who == self.seats[0] {
            Some(Side::First)
        } else if who == self.seats[1] {
            Some(Side::Second)
        } else {
            None
        }
    }

    pub fn clock_of(&self, side: Side) -> i64 {
        self.clock_ms[side.index()]
    }

    pub fn set_clock(&mut self, side: Side, ms: i64) {
        self.clock_ms[side.index()] = ms;
    }
}

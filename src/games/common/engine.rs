//! 通用回合制 2 人對戰引擎介面。各遊戲只需 impl `GameEngine`，
//! 大廳 / 桌位 / 配對 / 計時 / 斷線全由 `common::service` 共用。

use serde_json::Value;

/// 兩個座位（隨機分派給兩條連線）。`First` 先手。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    First,
    Second,
}

impl Side {
    pub fn opponent(self) -> Side {
        match self {
            Side::First => Side::Second,
            Side::Second => Side::First,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Side::First => 0,
            Side::Second => 1,
        }
    }
}

/// 對局狀態。`reason` 為引擎判定的結束原因字串（送前端 `game_over.reason`）。
pub enum GameStatus {
    Ongoing,
    Win { winner: Side, reason: &'static str },
    Draw { reason: &'static str },
}

/// 一步合法走子套用後的產物。
pub struct Applied {
    /// 併入 `move_made.data` 的欄位（物件，例如 `{from,to}` 或 `{col,row}`）。
    pub move_data: Value,
    /// move_made 之後額外推給雙方的事件（type, data），例如象棋 `("check", {...})`。
    pub extra: Vec<(&'static str, Value)>,
}

/// 各遊戲實作此 trait 即接上共用對戰框架。
pub trait GameEngine: Send + Sync + Sized + 'static {
    /// 遊戲識別字串，對應 WS 信封 `game` 欄、outbound 也帶回。
    const NAME: &'static str;
    const INITIAL_CLOCK_MS: i64 = 300_000; // 5:00
    const INCREMENT_MS: i64 = 30_000; // Fischer +30s

    fn initial() -> Self;

    /// 當前輪到哪個座位。
    fn turn(&self) -> Side;

    /// 座位對應的前端標籤（chess: red/black；gomoku: black/white；banqi: first/second）。
    /// 用於 match_found.color、move_made.turn、clock 鍵。靜態（不隨局面變）。
    fn side_label(side: Side) -> &'static str;

    /// 解析 + 驗證 + 套用一步。`Err(reason)` 時不可變更狀態（reason 為 illegal_move 字串）。
    /// 呼叫端已確認輪到 `mover`。
    fn try_move(&mut self, mover: Side, data: Option<&Value>) -> Result<Applied, String>;

    fn status(&self) -> GameStatus;
}

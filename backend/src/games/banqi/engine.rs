//! 暗棋（翻棋）規則引擎 — 純函式。4×8，象棋全套 32 子面朝下，翻開定色。
//!
//! 規則要點：
//! - 第一手必須翻子；翻子者執「翻出的顏色」，對手執另一色。
//! - 一般子吃子：相鄰一步，吃階 ≤ 己之敵子。例外：卒/兵可吃將/帥；將/帥不可吃卒/兵。同階可互吃。
//! - 炮：不吃子時相鄰一步走空格；吃子時同線隔「恰一個棋架」（任意距離），可吃任意階「已翻開」敵子。
//! - 勝負：某方棋子全被吃 / 輪到方無步可走 → 該方負。連續 N 手無吃子 → 和。
//! - 本實作房規：一般子與炮皆**不可吃面朝下**的子（僅吃已翻開敵子）。

use rand::seq::SliceRandom;

pub const COLS: i8 = 8;
pub const ROWS: i8 = 4;
const QUIET_DRAW: u32 = 60; // 連續無吃子半步數上限 → 和

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

impl Color {
    pub fn opponent(self) -> Color {
        match self {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Color::Red => "red",
            Color::Black => "black",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    King,
    Guard,
    Elephant,
    Rook,
    Horse,
    Cannon,
    Pawn,
}

impl Kind {
    pub fn rank(self) -> u8 {
        match self {
            Kind::King => 7,
            Kind::Guard => 6,
            Kind::Elephant => 5,
            Kind::Rook => 4,
            Kind::Horse => 3,
            Kind::Cannon => 2,
            Kind::Pawn => 1,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::King => "king",
            Kind::Guard => "guard",
            Kind::Elephant => "elephant",
            Kind::Rook => "rook",
            Kind::Horse => "horse",
            Kind::Cannon => "cannon",
            Kind::Pawn => "pawn",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Hidden(Piece),
    Up(Piece),
}

/// 哪一方先手由外層（seat）決定，引擎以 `Side` 表示 first/second。
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
}

#[derive(Clone)]
pub struct BanqiState {
    pub board: [[Cell; 8]; 4], // board[row][col]
    pub turn: Side,
    /// First 座位所執顏色，首次翻子後確定。
    pub first_color: Option<Color>,
    pub quiet: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Flip { col: i8, row: i8 },
    Move { from: (i8, i8), to: (i8, i8) },
}

/// 一步套用後的效果，供外層產生事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Flipped { col: i8, row: i8, piece: Piece },
    Moved { from: (i8, i8), to: (i8, i8), captured: Option<Piece> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Win { winner: Side, reason: &'static str },
    Draw,
}

fn full_deck() -> Vec<Piece> {
    use Kind::*;
    let counts = [(King, 1), (Guard, 2), (Elephant, 2), (Rook, 2), (Horse, 2), (Cannon, 2), (Pawn, 5)];
    let mut v = Vec::with_capacity(32);
    for color in [Color::Red, Color::Black] {
        for (kind, n) in counts {
            for _ in 0..n {
                v.push(Piece { color, kind });
            }
        }
    }
    v
}

/// 隨機洗牌的初始局面（全部面朝下，First 先行）。
pub fn initial_state() -> BanqiState {
    let mut deck = full_deck();
    deck.shuffle(&mut rand::thread_rng());
    from_deck(deck)
}

/// 由給定 32 子順序鋪盤（測試用，確定性）。
pub fn from_deck(deck: Vec<Piece>) -> BanqiState {
    assert_eq!(deck.len(), 32);
    let mut board = [[Cell::Empty; 8]; 4];
    let mut it = deck.into_iter();
    for row in 0..ROWS as usize {
        for col in 0..COLS as usize {
            board[row][col] = Cell::Hidden(it.next().unwrap());
        }
    }
    BanqiState {
        board,
        turn: Side::First,
        first_color: None,
        quiet: 0,
    }
}

fn in_bounds(col: i8, row: i8) -> bool {
    (0..COLS).contains(&col) && (0..ROWS).contains(&row)
}

fn cell(state: &BanqiState, col: i8, row: i8) -> Cell {
    if in_bounds(col, row) {
        state.board[row as usize][col as usize]
    } else {
        Cell::Empty
    }
}

pub fn color_of(state: &BanqiState, side: Side) -> Option<Color> {
    state.first_color.map(|c| match side {
        Side::First => c,
        Side::Second => c.opponent(),
    })
}

/// 吃子位階規則（炮另計）。
fn can_capture(att: Kind, tgt: Kind) -> bool {
    match (att, tgt) {
        (Kind::Pawn, Kind::King) => true,
        (Kind::King, Kind::Pawn) => false,
        _ => att.rank() >= tgt.rank(),
    }
}

/// 同線（含兩端外）之間的佔據格數。非同線回 None。
fn pieces_between(state: &BanqiState, from: (i8, i8), to: (i8, i8)) -> Option<u32> {
    let (fc, fr) = from;
    let (tc, tr) = to;
    if fc == tc {
        let (lo, hi) = (fr.min(tr), fr.max(tr));
        let mut n = 0;
        for r in (lo + 1)..hi {
            if !matches!(cell(state, fc, r), Cell::Empty) {
                n += 1;
            }
        }
        Some(n)
    } else if fr == tr {
        let (lo, hi) = (fc.min(tc), fc.max(tc));
        let mut n = 0;
        for c in (lo + 1)..hi {
            if !matches!(cell(state, c, fr), Cell::Empty) {
                n += 1;
            }
        }
        Some(n)
    } else {
        None
    }
}

/// 驗證一步走子（不含翻子），回 captured 棋子。假設 from 為 mover 已翻開己子。
fn validate_move(
    state: &BanqiState,
    piece: Piece,
    from: (i8, i8),
    to: (i8, i8),
) -> Result<Option<Piece>, &'static str> {
    if !in_bounds(to.0, to.1) {
        return Err("bad_coord");
    }
    let dc = (to.0 - from.0).abs();
    let dr = (to.1 - from.1).abs();

    if piece.kind == Kind::Cannon {
        // 相鄰走空格（不吃）
        if dc + dr == 1 {
            return match cell(state, to.0, to.1) {
                Cell::Empty => Ok(None),
                _ => Err("blocked"), // 相鄰非空：炮不可如此吃，須隔架
            };
        }
        // 隔一架吃同線敵子（任意距離）
        let between = pieces_between(state, from, to).ok_or("bad_cannon_move")?;
        if between != 1 {
            return Err("cannon_needs_screen");
        }
        return match cell(state, to.0, to.1) {
            Cell::Up(t) if t.color != piece.color => Ok(Some(t)),
            Cell::Up(_) => Err("occupied_friendly"),
            _ => Err("target_hidden"), // 空或面朝下：非合法炮吃
        };
    }

    // 一般子：相鄰一步
    if dc + dr != 1 {
        return Err("not_adjacent");
    }
    match cell(state, to.0, to.1) {
        Cell::Empty => Ok(None),
        Cell::Hidden(_) => Err("target_hidden"),
        Cell::Up(t) => {
            if t.color == piece.color {
                Err("occupied_friendly")
            } else if can_capture(piece.kind, t.kind) {
                Ok(Some(t))
            } else {
                Err("cannot_capture")
            }
        }
    }
}

/// 驗證 + 套用一動。Err 時不變更狀態。呼叫端已確認輪到 `mover`。
pub fn apply_action(
    state: &mut BanqiState,
    mover: Side,
    action: Action,
) -> Result<Effect, &'static str> {
    match action {
        Action::Flip { col, row } => {
            let piece = match cell(state, col, row) {
                Cell::Hidden(p) => p,
                Cell::Empty => return Err("not_hidden"),
                Cell::Up(_) => return Err("already_up"),
            };
            // 首翻定色
            if state.first_color.is_none() {
                state.first_color = Some(match mover {
                    Side::First => piece.color,
                    Side::Second => piece.color.opponent(),
                });
            }
            state.board[row as usize][col as usize] = Cell::Up(piece);
            state.quiet = state.quiet.saturating_add(1);
            state.turn = state.turn.opponent();
            Ok(Effect::Flipped { col, row, piece })
        }
        Action::Move { from, to } => {
            let color = color_of(state, mover).ok_or("no_color_yet")?;
            if !in_bounds(from.0, from.1) {
                return Err("bad_coord");
            }
            let piece = match cell(state, from.0, from.1) {
                Cell::Up(p) => p,
                Cell::Hidden(_) => return Err("not_revealed"),
                Cell::Empty => return Err("empty_from"),
            };
            if piece.color != color {
                return Err("not_your_piece");
            }
            let captured = validate_move(state, piece, from, to)?;
            // 套用
            state.board[from.1 as usize][from.0 as usize] = Cell::Empty;
            state.board[to.1 as usize][to.0 as usize] = Cell::Up(piece);
            if captured.is_some() {
                state.quiet = 0;
            } else {
                state.quiet = state.quiet.saturating_add(1);
            }
            state.turn = state.turn.opponent();
            Ok(Effect::Moved { from, to, captured })
        }
    }
}

/// 場上某色剩餘子數（含面朝下）。
fn count_color(state: &BanqiState, color: Color) -> u32 {
    let mut n = 0;
    for row in 0..ROWS {
        for col in 0..COLS {
            match cell(state, col, row) {
                Cell::Hidden(p) | Cell::Up(p) if p.color == color => n += 1,
                _ => {}
            }
        }
    }
    n
}

fn any_hidden(state: &BanqiState) -> bool {
    state
        .board
        .iter()
        .flatten()
        .any(|c| matches!(c, Cell::Hidden(_)))
}

/// `side` 是否還有任何合法動作（翻子或走子）。
fn has_action(state: &BanqiState, side: Side) -> bool {
    if any_hidden(state) {
        return true; // 永遠可翻
    }
    let Some(color) = color_of(state, side) else {
        return false;
    };
    for row in 0..ROWS {
        for col in 0..COLS {
            let Cell::Up(p) = cell(state, col, row) else {
                continue;
            };
            if p.color != color {
                continue;
            }
            if p.kind == Kind::Cannon {
                // 相鄰空格
                for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if matches!(cell(state, col + dc, row + dr), Cell::Empty) {
                        return true;
                    }
                }
                // 隔架吃：掃同行同列
                for r in 0..ROWS {
                    if r != row && validate_move(state, p, (col, row), (col, r)).is_ok() {
                        return true;
                    }
                }
                for c in 0..COLS {
                    if c != col && validate_move(state, p, (col, row), (c, row)).is_ok() {
                        return true;
                    }
                }
            } else {
                for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if validate_move(state, p, (col, row), (col + dc, row + dr)).is_ok() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn status(state: &BanqiState) -> Outcome {
    // 未定色（尚無翻子）→ 進行中
    let Some(_) = state.first_color else {
        return Outcome::Continue;
    };
    let first_alive = count_color(state, color_of(state, Side::First).unwrap());
    let second_alive = count_color(state, color_of(state, Side::Second).unwrap());
    match (first_alive, second_alive) {
        (0, 0) => return Outcome::Draw,
        (0, _) => return Outcome::Win { winner: Side::Second, reason: "elimination" },
        (_, 0) => return Outcome::Win { winner: Side::First, reason: "elimination" },
        _ => {}
    }
    // 輪到方無步可走 → 判負
    if !has_action(state, state.turn) {
        return Outcome::Win { winner: state.turn.opponent(), reason: "no_moves" };
    }
    if state.quiet >= QUIET_DRAW {
        return Outcome::Draw;
    }
    Outcome::Continue
}

#[cfg(test)]
mod tests;

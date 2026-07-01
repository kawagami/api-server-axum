//! 西洋棋核心型別 — 純資料。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opponent(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

/// `[col, row]`：col 0–7（a–h），row 0–7（白方底線 row 0）。
pub type Coord = (i8, i8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    /// 升變目標（僅兵到底線時）。
    pub promo: Option<PieceKind>,
}

pub type Board = [[Option<Piece>; 8]; 8];

#[derive(Debug, Clone)]
pub struct State {
    pub board: Board,
    pub turn: Color,
    /// 易位權 [WK, WQ, BK, BQ]。
    pub castling: [bool; 4],
    /// 過路兵目標格（可被斜吃的空格）。
    pub ep: Option<Coord>,
    /// 無吃子 / 無兵走 的半步數（50 步和）。
    pub halfmove: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Castle {
    King,
    Queen,
}

/// 套用走步後供前端渲染的特殊資訊。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApplyInfo {
    pub castle: Option<Castle>,
    /// 過路兵被吃的格（該兵不在 `to` 上）。
    pub ep_capture: Option<Coord>,
    pub promo: Option<PieceKind>,
}

//! 象棋核心型別 — 純資料，零 WS / 零 IO 依賴。

use serde::Serialize;

/// 紅（下）/ 黑（上）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Red,
    Black,
}

impl Side {
    pub fn opponent(self) -> Side {
        match self {
            Side::Red => Side::Black,
            Side::Black => Side::Red,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Side::Red => "red",
            Side::Black => "black",
        }
    }
}

/// 棋子種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Rook,     // 車 / 俥
    Horse,    // 馬 / 傌
    Cannon,   // 炮 / 砲
    Elephant, // 象 / 相
    Advisor,  // 士 / 仕
    General,  // 將 / 帥
    Soldier,  // 兵 / 卒
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub side: Side,
}

impl Piece {
    pub fn new(kind: PieceKind, side: Side) -> Self {
        Piece { kind, side }
    }
}

/// 絕對座標：col 0–8（左→右），row 0–9（下→上）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub col: i8,
    pub row: i8,
}

impl Square {
    pub fn new(col: i8, row: i8) -> Self {
        Square { col, row }
    }

    pub fn in_bounds(self) -> bool {
        (0..9).contains(&self.col) && (0..10).contains(&self.row)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

/// 9 路 × 10 線盤面，索引 `board[row][col]`。
pub type Board = [[Option<Piece>; 9]; 10];

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub turn: Side,
    /// 連續無吃子半步數（兩半步＝一回合，120 半步＝60 回合判和）。
    pub halfmove_no_capture: u8,
}

/// 對局結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ongoing,
    Checkmate { winner: Side },
    /// 困斃：被困方判負。
    Stalemate { loser: Side },
    Draw,
}

/// 非法走步原因；`code()` 為傳給前端的字串。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IllegalReason {
    NotYourTurn,
    NoPiece,
    WrongPiece,
    CaptureOwn,
    BadMove,
    BlockedPath,
    BadHorseLeg,
    BadElephantEye,
    CrossRiver,
    OutOfPalace,
    FlyingGeneral,
    LeavesKingInCheck,
}

impl IllegalReason {
    pub fn code(self) -> &'static str {
        match self {
            IllegalReason::NotYourTurn => "NotYourTurn",
            IllegalReason::NoPiece => "NoPiece",
            IllegalReason::WrongPiece => "WrongPiece",
            IllegalReason::CaptureOwn => "CaptureOwn",
            IllegalReason::BadMove => "BadMove",
            IllegalReason::BlockedPath => "BlockedPath",
            IllegalReason::BadHorseLeg => "BadHorseLeg",
            IllegalReason::BadElephantEye => "BadElephantEye",
            IllegalReason::CrossRiver => "CrossRiver",
            IllegalReason::OutOfPalace => "OutOfPalace",
            IllegalReason::FlyingGeneral => "FlyingGeneral",
            IllegalReason::LeavesKingInCheck => "LeavesKingInCheck",
        }
    }
}

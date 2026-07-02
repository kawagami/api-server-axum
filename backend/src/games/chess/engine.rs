//! 象棋規則引擎 — 純函式，可單測。標準走法 + 將軍/將死 + 飛將 + 困斃 + 60 回合和。

use super::types::{Board, GameState, IllegalReason, Move, Piece, PieceKind, Side, Square, Status};

const N_NO_CAPTURE_DRAW: u8 = 120; // 60 回合 = 120 半步

/// 標準開局局面（紅先行）。
pub fn initial_state() -> GameState {
    let mut board: Board = Default::default();
    let mut place = |col: i8, row: i8, kind: PieceKind, side: Side| {
        board[row as usize][col as usize] = Some(Piece::new(kind, side));
    };

    use PieceKind::*;
    use Side::*;

    // 紅方（下，row 0–4）
    for c in [0, 8] {
        place(c, 0, Rook, Red);
    }
    for c in [1, 7] {
        place(c, 0, Horse, Red);
    }
    for c in [2, 6] {
        place(c, 0, Elephant, Red);
    }
    for c in [3, 5] {
        place(c, 0, Advisor, Red);
    }
    place(4, 0, General, Red);
    for c in [1, 7] {
        place(c, 2, Cannon, Red);
    }
    for c in [0, 2, 4, 6, 8] {
        place(c, 3, Soldier, Red);
    }

    // 黑方（上，row 5–9）
    for c in [0, 8] {
        place(c, 9, Rook, Black);
    }
    for c in [1, 7] {
        place(c, 9, Horse, Black);
    }
    for c in [2, 6] {
        place(c, 9, Elephant, Black);
    }
    for c in [3, 5] {
        place(c, 9, Advisor, Black);
    }
    place(4, 9, General, Black);
    for c in [1, 7] {
        place(c, 7, Cannon, Black);
    }
    for c in [0, 2, 4, 6, 8] {
        place(c, 6, Soldier, Black);
    }

    GameState {
        board,
        turn: Side::Red,
        halfmove_no_capture: 0,
    }
}

fn piece_at(board: &Board, sq: Square) -> Option<Piece> {
    if !sq.in_bounds() {
        return None;
    }
    board[sq.row as usize][sq.col as usize]
}

fn in_palace(side: Side, sq: Square) -> bool {
    let col_ok = (3..=5).contains(&sq.col);
    let row_ok = match side {
        Side::Red => (0..=2).contains(&sq.row),
        Side::Black => (7..=9).contains(&sq.row),
    };
    col_ok && row_ok
}

/// 紅子過河：row ≥ 5；黑子過河：row ≤ 4。
fn crossed_river(side: Side, row: i8) -> bool {
    match side {
        Side::Red => row >= 5,
        Side::Black => row <= 4,
    }
}

/// 直線（同行或同列）兩格之間的棋子數（不含兩端）。回傳 None 表示非直線。
fn pieces_between(board: &Board, from: Square, to: Square) -> Option<u32> {
    if from.col == to.col {
        let (lo, hi) = (from.row.min(to.row), from.row.max(to.row));
        let mut count = 0;
        for r in (lo + 1)..hi {
            if board[r as usize][from.col as usize].is_some() {
                count += 1;
            }
        }
        Some(count)
    } else if from.row == to.row {
        let (lo, hi) = (from.col.min(to.col), from.col.max(to.col));
        let mut count = 0;
        for c in (lo + 1)..hi {
            if board[from.row as usize][c as usize].is_some() {
                count += 1;
            }
        }
        Some(count)
    } else {
        None
    }
}

/// 走步形狀檢查（含蹩馬腳/塞象眼/翻山/過河/九宮），不含送將判定。
/// 假設 from 有己方子、to 已通過越界與吃己方子檢查。
fn validate_geometry(board: &Board, mv: Move, piece: Piece) -> Result<(), IllegalReason> {
    let from = mv.from;
    let to = mv.to;
    let dc = to.col - from.col;
    let dr = to.row - from.row;

    match piece.kind {
        PieceKind::Rook => {
            if dc != 0 && dr != 0 {
                return Err(IllegalReason::BadMove);
            }
            match pieces_between(board, from, to) {
                Some(0) => Ok(()),
                _ => Err(IllegalReason::BlockedPath),
            }
        }
        PieceKind::Cannon => {
            if dc != 0 && dr != 0 {
                return Err(IllegalReason::BadMove);
            }
            let between = pieces_between(board, from, to).unwrap_or(99);
            let capturing = piece_at(board, to).is_some();
            if capturing {
                // 吃子需恰一個炮架
                if between == 1 {
                    Ok(())
                } else {
                    Err(IllegalReason::BlockedPath)
                }
            } else {
                // 不吃子路徑須空
                if between == 0 {
                    Ok(())
                } else {
                    Err(IllegalReason::BlockedPath)
                }
            }
        }
        PieceKind::Horse => {
            // 日字：(±1,±2) 或 (±2,±1)
            let leg = match (dc.abs(), dr.abs()) {
                (1, 2) => Square::new(from.col, from.row + dr.signum()), // 先直走一格
                (2, 1) => Square::new(from.col + dc.signum(), from.row), // 先橫走一格
                _ => return Err(IllegalReason::BadMove),
            };
            if piece_at(board, leg).is_some() {
                return Err(IllegalReason::BadHorseLeg); // 蹩馬腳
            }
            Ok(())
        }
        PieceKind::Elephant => {
            if dc.abs() != 2 || dr.abs() != 2 {
                return Err(IllegalReason::BadMove);
            }
            if crossed_river(piece.side, to.row) {
                return Err(IllegalReason::CrossRiver); // 象不可過河
            }
            let eye = Square::new(from.col + dc.signum(), from.row + dr.signum());
            if piece_at(board, eye).is_some() {
                return Err(IllegalReason::BadElephantEye); // 塞象眼
            }
            Ok(())
        }
        PieceKind::Advisor => {
            if dc.abs() != 1 || dr.abs() != 1 {
                return Err(IllegalReason::BadMove);
            }
            if !in_palace(piece.side, to) {
                return Err(IllegalReason::OutOfPalace);
            }
            Ok(())
        }
        PieceKind::General => {
            if (dc.abs() + dr.abs()) != 1 {
                return Err(IllegalReason::BadMove);
            }
            if !in_palace(piece.side, to) {
                return Err(IllegalReason::OutOfPalace);
            }
            Ok(())
        }
        PieceKind::Soldier => {
            let forward = match piece.side {
                Side::Red => 1,
                Side::Black => -1,
            };
            if dc == 0 && dr == forward {
                // 前進一格永遠合法
                Ok(())
            } else if dr == 0 && dc.abs() == 1 && crossed_river(piece.side, from.row) {
                // 過河後可左右一格
                Ok(())
            } else {
                Err(IllegalReason::BadMove)
            }
        }
    }
}

fn find_general(board: &Board, side: Side) -> Option<Square> {
    for (r, row) in board.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            if let Some(p) = cell {
                if p.kind == PieceKind::General && p.side == side {
                    return Some(Square::new(c as i8, r as i8));
                }
            }
        }
    }
    None
}

/// `by_side` 是否攻擊 `target`（含飛將線）。target 假設有敵子或為將位。
fn is_attacked(board: &Board, target: Square, by_side: Side) -> bool {
    for r in 0..10 {
        for c in 0..9 {
            let from = Square::new(c as i8, r as i8);
            let Some(p) = board[r][c] else { continue };
            if p.side != by_side {
                continue;
            }
            if p.kind == PieceKind::General {
                // 將不靠一般走法攻擊；飛將線另外處理
                continue;
            }
            // 攻擊 = 該子能以吃子走法落在 target（target 此時被視為敵子）
            if validate_geometry(board, Move { from, to: target }, p).is_ok() {
                return true;
            }
        }
    }
    // 飛將：對方將與 target 同 column 且中間無子
    if let Some(enemy_general) = find_general(board, by_side) {
        if enemy_general.col == target.col
            && pieces_between(board, enemy_general, target) == Some(0)
        {
            return true;
        }
    }
    false
}

pub fn is_in_check(state: &GameState, side: Side) -> bool {
    match find_general(&state.board, side) {
        Some(g) => is_attacked(&state.board, g, side.opponent()),
        None => true, // 將已被吃，視為最壞情況
    }
}

/// 套用走步（假設已合法）：移子、更新無吃子計數、切換行棋方。
pub fn apply(state: &mut GameState, mv: Move) {
    let captured = piece_at(&state.board, mv.to).is_some();
    let piece = state.board[mv.from.row as usize][mv.from.col as usize].take();
    state.board[mv.to.row as usize][mv.to.col as usize] = piece;
    if captured {
        state.halfmove_no_capture = 0;
    } else {
        state.halfmove_no_capture = state.halfmove_no_capture.saturating_add(1);
    }
    state.turn = state.turn.opponent();
}

/// 完整合法性檢查（含送將/飛將）。
pub fn is_legal(state: &GameState, mv: Move) -> Result<(), IllegalReason> {
    if !mv.from.in_bounds() || !mv.to.in_bounds() {
        return Err(IllegalReason::BadMove);
    }
    if mv.from == mv.to {
        return Err(IllegalReason::BadMove);
    }
    let piece = piece_at(&state.board, mv.from).ok_or(IllegalReason::NoPiece)?;
    if piece.side != state.turn {
        return Err(IllegalReason::WrongPiece);
    }
    if let Some(target) = piece_at(&state.board, mv.to) {
        if target.side == piece.side {
            return Err(IllegalReason::CaptureOwn);
        }
    }
    validate_geometry(&state.board, mv, piece)?;

    // 模擬走步後，己方將不可被將（含飛將對臉）
    let mut next = state.clone();
    apply(&mut next, mv);
    if is_in_check(&next, piece.side) {
        // 區分飛將與一般送將：走完後兩將對臉 → FlyingGeneral
        if generals_face(&next.board) {
            return Err(IllegalReason::FlyingGeneral);
        }
        return Err(IllegalReason::LeavesKingInCheck);
    }
    Ok(())
}

/// 兩將同 column 且中間無子。
fn generals_face(board: &Board) -> bool {
    let (Some(r), Some(b)) = (find_general(board, Side::Red), find_general(board, Side::Black))
    else {
        return false;
    };
    r.col == b.col && pieces_between(board, r, b) == Some(0)
}

/// 單子所有合法目標（已過濾送將），供前端 UX 提示。
pub fn legal_moves(state: &GameState, sq: Square) -> Vec<Square> {
    let Some(piece) = piece_at(&state.board, sq) else {
        return Vec::new();
    };
    if piece.side != state.turn {
        return Vec::new();
    }
    let mut out = Vec::new();
    for r in 0..10 {
        for c in 0..9 {
            let to = Square::new(c as i8, r as i8);
            if is_legal(state, Move { from: sq, to }).is_ok() {
                out.push(to);
            }
        }
    }
    out
}

/// 行棋方是否還有任何合法步。
fn has_any_legal_move(state: &GameState) -> bool {
    for r in 0..10 {
        for c in 0..9 {
            if let Some(p) = state.board[r][c] {
                if p.side == state.turn {
                    let from = Square::new(c as i8, r as i8);
                    if !legal_moves(state, from).is_empty() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn game_status(state: &GameState) -> Status {
    if !has_any_legal_move(state) {
        if is_in_check(state, state.turn) {
            // 行棋方被將死 → 對方勝
            return Status::Checkmate {
                winner: state.turn.opponent(),
            };
        }
        // 困斃：行棋方判負
        return Status::Stalemate { loser: state.turn };
    }
    if state.halfmove_no_capture >= N_NO_CAPTURE_DRAW {
        return Status::Draw;
    }
    Status::Ongoing
}

#[cfg(test)]
mod tests;

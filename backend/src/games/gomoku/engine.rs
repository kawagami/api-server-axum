//! 五子棋規則引擎 — 純函式，可單測。15×15，黑先，五子（含以上）連線勝，滿盤和。

pub const SIZE: i8 = 15;
const WIN_LEN: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stone {
    Black,
    White,
}

impl Stone {
    pub fn opponent(self) -> Stone {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Win(Stone),
    Draw,
}

#[derive(Clone)]
pub struct GomokuState {
    pub board: [[Option<Stone>; 15]; 15],
    pub turn: Stone,
    pub placed: u32,
}

pub fn initial_state() -> GomokuState {
    GomokuState {
        board: [[None; 15]; 15],
        turn: Stone::Black,
        placed: 0,
    }
}

fn in_bounds(col: i8, row: i8) -> bool {
    (0..SIZE).contains(&col) && (0..SIZE).contains(&row)
}

fn at(state: &GomokuState, col: i8, row: i8) -> Option<Stone> {
    if in_bounds(col, row) {
        state.board[row as usize][col as usize]
    } else {
        None
    }
}

/// 落子合法性：界內且空格。
pub fn is_legal(state: &GomokuState, col: i8, row: i8) -> Result<(), &'static str> {
    if !in_bounds(col, row) {
        return Err("bad_coord");
    }
    if state.board[row as usize][col as usize].is_some() {
        return Err("occupied");
    }
    Ok(())
}

/// 套用落子（假設已合法）：放當前 turn 的子、計數、換手。
pub fn apply(state: &mut GomokuState, col: i8, row: i8) {
    state.board[row as usize][col as usize] = Some(state.turn);
    state.placed += 1;
    state.turn = state.turn.opponent();
}

/// 任一方達 5（含以上）連線即勝；滿盤判和。與當前 turn 無關（掃整盤）。
pub fn status(state: &GomokuState) -> Outcome {
    const DIRS: [(i8, i8); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
    for row in 0..SIZE {
        for col in 0..SIZE {
            let Some(stone) = at(state, col, row) else {
                continue;
            };
            for (dc, dr) in DIRS {
                // 只從連線起點計（前一格非同色），避免重複計
                if at(state, col - dc, row - dr) == Some(stone) {
                    continue;
                }
                let mut len = 0;
                let (mut c, mut r) = (col, row);
                while at(state, c, r) == Some(stone) {
                    len += 1;
                    c += dc;
                    r += dr;
                }
                if len >= WIN_LEN {
                    return Outcome::Win(stone);
                }
            }
        }
    }
    if state.placed as i32 >= (SIZE as i32) * (SIZE as i32) {
        Outcome::Draw
    } else {
        Outcome::Continue
    }
}

#[cfg(test)]
mod tests;

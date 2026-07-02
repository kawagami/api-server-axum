//! 圍棋規則引擎 — 純函式，零 WS 依賴。提子（無氣移除）/ 禁自殺 / 簡單劫 / 虛手 /
//! 雙虛手終局 + 數子（area scoring）+ 貼目。19 路，komi 7.5。

pub const SIZE: i8 = 19;
const KOMI_X2: i32 = 15; // 7.5 * 2（以倍數保持整數，杜絕和局）

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

pub type Board = [[Option<Stone>; SIZE as usize]; SIZE as usize];

#[derive(Debug, Clone)]
pub struct GoState {
    pub board: Board,
    pub turn: Stone,
    /// 劫爭禁著點（對手下一手不可下此點）。
    pub ko: Option<(i8, i8)>,
    /// 連續虛手數，達 2 終局。
    pub passes: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    OutOfBounds,
    Occupied,
    Ko,
    Suicide,
}

impl Reason {
    pub fn as_str(self) -> &'static str {
        match self {
            Reason::OutOfBounds => "out_of_bounds",
            Reason::Occupied => "occupied",
            Reason::Ko => "ko",
            Reason::Suicide => "suicide",
        }
    }
}

pub fn initial_state() -> GoState {
    GoState {
        board: [[None; SIZE as usize]; SIZE as usize],
        turn: Stone::Black, // 黑先
        ko: None,
        passes: 0,
    }
}

fn in_bounds(c: i8, r: i8) -> bool {
    (0..SIZE).contains(&c) && (0..SIZE).contains(&r)
}

fn neighbors(c: i8, r: i8) -> Vec<(i8, i8)> {
    [(c + 1, r), (c - 1, r), (c, r + 1), (c, r - 1)]
        .into_iter()
        .filter(|&(nc, nr)| in_bounds(nc, nr))
        .collect()
}

/// 回傳 (c,r) 所屬同色連通塊的所有座標與其氣數。
fn group_and_liberties(board: &Board, c: i8, r: i8) -> (Vec<(i8, i8)>, usize) {
    let color = board[r as usize][c as usize];
    let mut stack = vec![(c, r)];
    let mut seen = vec![(c, r)];
    let mut libs = std::collections::HashSet::new();
    while let Some((cc, rr)) = stack.pop() {
        for (nc, nr) in neighbors(cc, rr) {
            match board[nr as usize][nc as usize] {
                None => {
                    libs.insert((nc, nr));
                }
                x if x == color && !seen.contains(&(nc, nr)) => {
                    seen.push((nc, nr));
                    stack.push((nc, nr));
                }
                _ => {}
            }
        }
    }
    (seen, libs.len())
}

/// 在 board 落子並提走無氣的對方連通塊，回傳被提座標。假設 (c,r) 原為空。
fn place_and_capture(board: &mut Board, c: i8, r: i8, color: Stone) -> Vec<(i8, i8)> {
    board[r as usize][c as usize] = Some(color);
    let mut captured = Vec::new();
    for (nc, nr) in neighbors(c, r) {
        if board[nr as usize][nc as usize] == Some(color.opponent()) {
            let (group, libs) = group_and_liberties(board, nc, nr);
            if libs == 0 {
                for (gc, gr) in group {
                    board[gr as usize][gc as usize] = None;
                    captured.push((gc, gr));
                }
            }
        }
    }
    captured
}

pub fn is_legal(state: &GoState, c: i8, r: i8) -> Result<(), Reason> {
    if !in_bounds(c, r) {
        return Err(Reason::OutOfBounds);
    }
    if state.board[r as usize][c as usize].is_some() {
        return Err(Reason::Occupied);
    }
    if state.ko == Some((c, r)) {
        return Err(Reason::Ko);
    }
    // 模擬：落子提子後自group須有氣（否則自殺）
    let mut board = state.board;
    place_and_capture(&mut board, c, r, state.turn);
    let (_, libs) = group_and_liberties(&board, c, r);
    if libs == 0 {
        return Err(Reason::Suicide);
    }
    Ok(())
}

/// 落子（假設已合法）。回傳被提座標。更新劫點、清虛手數、換手。
pub fn apply(state: &mut GoState, c: i8, r: i8) -> Vec<(i8, i8)> {
    let color = state.turn;
    let captured = place_and_capture(&mut state.board, c, r, color);
    // 簡單劫：恰提一子，且落子成單子且僅一氣 → 該提點設為劫禁
    let (group, libs) = group_and_liberties(&state.board, c, r);
    state.ko = if captured.len() == 1 && group.len() == 1 && libs == 1 {
        Some(captured[0])
    } else {
        None
    };
    state.passes = 0;
    state.turn = color.opponent();
    captured
}

/// 虛手。換手、累計虛手、清劫點。
pub fn pass(state: &mut GoState) {
    state.passes += 1;
    state.ko = None;
    state.turn = state.turn.opponent();
}

/// 數子（area scoring）：各方 = 己方棋子數 + 只被己方圍的空白地。回傳 (黑, 白)。
pub fn score(board: &Board) -> (i32, i32) {
    let mut black = 0;
    let mut white = 0;
    let mut visited = [[false; SIZE as usize]; SIZE as usize];

    for r in 0..SIZE {
        for c in 0..SIZE {
            match board[r as usize][c as usize] {
                Some(Stone::Black) => black += 1,
                Some(Stone::White) => white += 1,
                None => {
                    if visited[r as usize][c as usize] {
                        continue;
                    }
                    // 泛洪整塊空白，記錄邊界顏色
                    let mut stack = vec![(c, r)];
                    let mut region = Vec::new();
                    let mut touch_black = false;
                    let mut touch_white = false;
                    visited[r as usize][c as usize] = true;
                    while let Some((cc, rr)) = stack.pop() {
                        region.push((cc, rr));
                        for (nc, nr) in neighbors(cc, rr) {
                            match board[nr as usize][nc as usize] {
                                Some(Stone::Black) => touch_black = true,
                                Some(Stone::White) => touch_white = true,
                                None => {
                                    if !visited[nr as usize][nc as usize] {
                                        visited[nr as usize][nc as usize] = true;
                                        stack.push((nc, nr));
                                    }
                                }
                            }
                        }
                    }
                    if touch_black && !touch_white {
                        black += region.len() as i32;
                    } else if touch_white && !touch_black {
                        white += region.len() as i32;
                    } // 兩色皆鄰 = 中立，不計
                }
            }
        }
    }
    (black, white)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    /// 雙虛手終局，依數子 + 貼目判勝。
    Win(Stone),
}

pub fn status(state: &GoState) -> Outcome {
    if state.passes < 2 {
        return Outcome::Continue;
    }
    let (b, w) = score(&state.board);
    // 倍數比較：白方 +komi。KOMI_X2 為奇數 → 無平手
    if b * 2 > w * 2 + KOMI_X2 {
        Outcome::Win(Stone::Black)
    } else {
        Outcome::Win(Stone::White)
    }
}

#[cfg(test)]
mod tests;

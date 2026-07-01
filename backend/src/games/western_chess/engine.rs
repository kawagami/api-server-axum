//! 西洋棋規則引擎 — 純函式，零 WS 依賴。標準走法 + 王車易位 + 吃過路兵 + 升變
//! + 將軍/將死/逼和 + 50 步和 + 子力不足和。

use super::types::{ApplyInfo, Castle, Color, Move, Piece, PieceKind, State};

const N_HALFMOVE_DRAW: u32 = 100; // 50 步（雙方各 50）無吃子/無兵走 → 和

pub fn initial_state() -> State {
    use Color::*;
    use PieceKind::*;
    let mut board = [[None; 8]; 8];
    let back = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
    for (c, &kind) in back.iter().enumerate() {
        board[0][c] = Some(Piece { kind, color: White });
        board[7][c] = Some(Piece { kind, color: Black });
        board[1][c] = Some(Piece { kind: Pawn, color: White });
        board[6][c] = Some(Piece { kind: Pawn, color: Black });
    }
    State {
        board,
        turn: White,
        castling: [true; 4], // WK, WQ, BK, BQ
        ep: None,
        halfmove: 0,
    }
}

pub fn turn(state: &State) -> Color {
    state.turn
}

fn at(state: &State, c: i8, r: i8) -> Option<Piece> {
    if (0..8).contains(&c) && (0..8).contains(&r) {
        state.board[r as usize][c as usize]
    } else {
        None
    }
}

fn find_king(state: &State, color: Color) -> Option<(i8, i8)> {
    for r in 0..8 {
        for c in 0..8 {
            if let Some(p) = state.board[r][c] {
                if p.kind == PieceKind::King && p.color == color {
                    return Some((c as i8, r as i8));
                }
            }
        }
    }
    None
}

const KNIGHT: [(i8, i8); 8] = [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)];
const DIAG: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ORTHO: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// `sq` 是否被 `by` 方任一子攻擊（含兵的斜吃、王的鄰接，不含王車易位）。
pub fn is_attacked(state: &State, sq: (i8, i8), by: Color) -> bool {
    let (c, r) = sq;
    // 兵：by 方兵從「其前進方向」斜攻 sq → 兵位於 sq 的後方斜角
    let pawn_dir = match by {
        Color::White => 1,
        Color::Black => -1,
    };
    for dc in [-1, 1] {
        if let Some(p) = at(state, c + dc, r - pawn_dir) {
            if p.color == by && p.kind == PieceKind::Pawn {
                return true;
            }
        }
    }
    // 馬
    for (dc, dr) in KNIGHT {
        if let Some(p) = at(state, c + dc, r + dr) {
            if p.color == by && p.kind == PieceKind::Knight {
                return true;
            }
        }
    }
    // 王
    for dc in -1..=1 {
        for dr in -1..=1 {
            if (dc, dr) == (0, 0) {
                continue;
            }
            if let Some(p) = at(state, c + dc, r + dr) {
                if p.color == by && p.kind == PieceKind::King {
                    return true;
                }
            }
        }
    }
    // 直線（車/后）
    for (dc, dr) in ORTHO {
        if slide_hits(state, c, r, dc, dr, by, PieceKind::Rook) {
            return true;
        }
    }
    // 斜線（象/后）
    for (dc, dr) in DIAG {
        if slide_hits(state, c, r, dc, dr, by, PieceKind::Bishop) {
            return true;
        }
    }
    false
}

/// 從 (c,r) 沿 (dc,dr) 滑動，遇到的第一子若為 by 方且為 `line_kind` 或 Queen → 命中。
fn slide_hits(state: &State, c: i8, r: i8, dc: i8, dr: i8, by: Color, line_kind: PieceKind) -> bool {
    let (mut cc, mut rr) = (c + dc, r + dr);
    while (0..8).contains(&cc) && (0..8).contains(&rr) {
        if let Some(p) = state.board[rr as usize][cc as usize] {
            return p.color == by && (p.kind == line_kind || p.kind == PieceKind::Queen);
        }
        cc += dc;
        rr += dr;
    }
    false
}

pub fn is_in_check(state: &State, color: Color) -> bool {
    match find_king(state, color) {
        Some(k) => is_attacked(state, k, color.opponent()),
        None => true,
    }
}

/// 產生行棋方所有「偽合法」走步（含易位/升變/過路兵），未過濾送將。
fn pseudo_moves(state: &State) -> Vec<Move> {
    let mut out = Vec::new();
    let me = state.turn;
    for r in 0..8i8 {
        for c in 0..8i8 {
            let Some(p) = at(state, c, r) else { continue };
            if p.color != me {
                continue;
            }
            match p.kind {
                PieceKind::Pawn => pawn_moves(state, c, r, me, &mut out),
                PieceKind::Knight => {
                    for (dc, dr) in KNIGHT {
                        push_if_target(state, c, r, c + dc, r + dr, me, &mut out);
                    }
                }
                PieceKind::King => {
                    for dc in -1..=1 {
                        for dr in -1..=1 {
                            if (dc, dr) != (0, 0) {
                                push_if_target(state, c, r, c + dc, r + dr, me, &mut out);
                            }
                        }
                    }
                    castling_moves(state, c, r, me, &mut out);
                }
                PieceKind::Rook => slide_moves(state, c, r, me, &ORTHO, &mut out),
                PieceKind::Bishop => slide_moves(state, c, r, me, &DIAG, &mut out),
                PieceKind::Queen => {
                    slide_moves(state, c, r, me, &ORTHO, &mut out);
                    slide_moves(state, c, r, me, &DIAG, &mut out);
                }
            }
        }
    }
    out
}

fn push_if_target(state: &State, fc: i8, fr: i8, tc: i8, tr: i8, me: Color, out: &mut Vec<Move>) {
    if !(0..8).contains(&tc) || !(0..8).contains(&tr) {
        return;
    }
    match at(state, tc, tr) {
        Some(p) if p.color == me => {} // 自己子，不可
        _ => out.push(Move { from: (fc, fr), to: (tc, tr), promo: None }),
    }
}

fn slide_moves(state: &State, c: i8, r: i8, me: Color, dirs: &[(i8, i8)], out: &mut Vec<Move>) {
    for &(dc, dr) in dirs {
        let (mut cc, mut rr) = (c + dc, r + dr);
        while (0..8).contains(&cc) && (0..8).contains(&rr) {
            match at(state, cc, rr) {
                None => out.push(Move { from: (c, r), to: (cc, rr), promo: None }),
                Some(p) => {
                    if p.color != me {
                        out.push(Move { from: (c, r), to: (cc, rr), promo: None });
                    }
                    break;
                }
            }
            cc += dc;
            rr += dr;
        }
    }
}

fn pawn_moves(state: &State, c: i8, r: i8, me: Color, out: &mut Vec<Move>) {
    let dir = match me {
        Color::White => 1,
        Color::Black => -1,
    };
    let start_rank = if me == Color::White { 1 } else { 6 };
    let promo_rank = if me == Color::White { 7 } else { 0 };

    let push = |from: (i8, i8), to: (i8, i8), out: &mut Vec<Move>| {
        if to.1 == promo_rank {
            for k in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                out.push(Move { from, to, promo: Some(k) });
            }
        } else {
            out.push(Move { from, to, promo: None });
        }
    };

    // 前進一格
    if at(state, c, r + dir).is_none() {
        push((c, r), (c, r + dir), out);
        // 前進兩格
        if r == start_rank && at(state, c, r + 2 * dir).is_none() {
            out.push(Move { from: (c, r), to: (c, r + 2 * dir), promo: None });
        }
    }
    // 斜吃 + 過路兵
    for dc in [-1, 1] {
        let (tc, tr) = (c + dc, r + dir);
        if !(0..8).contains(&tc) || !(0..8).contains(&tr) {
            continue;
        }
        match at(state, tc, tr) {
            Some(p) if p.color != me => push((c, r), (tc, tr), out),
            _ => {
                if state.ep == Some((tc, tr)) {
                    out.push(Move { from: (c, r), to: (tc, tr), promo: None });
                }
            }
        }
    }
}

fn castling_moves(state: &State, c: i8, r: i8, me: Color, out: &mut Vec<Move>) {
    // 王必須在初始格且不在被將狀態
    let home_r = if me == Color::White { 0 } else { 7 };
    if (c, r) != (4, home_r) || is_in_check(state, me) {
        return;
    }
    let (kside, qside) = match me {
        Color::White => (0, 1),
        Color::Black => (2, 3),
    };
    let opp = me.opponent();
    // 王翼：f,g 空，王經 e->f->g 不被攻擊，h 為車
    if state.castling[kside]
        && at(state, 5, home_r).is_none()
        && at(state, 6, home_r).is_none()
        && !is_attacked(state, (5, home_r), opp)
        && !is_attacked(state, (6, home_r), opp)
    {
        out.push(Move { from: (4, home_r), to: (6, home_r), promo: None });
    }
    // 后翼：b,c,d 空，王經 e->d->c 不被攻擊
    if state.castling[qside]
        && at(state, 1, home_r).is_none()
        && at(state, 2, home_r).is_none()
        && at(state, 3, home_r).is_none()
        && !is_attacked(state, (3, home_r), opp)
        && !is_attacked(state, (2, home_r), opp)
    {
        out.push(Move { from: (4, home_r), to: (2, home_r), promo: None });
    }
}

/// 套用走步（假設已合法），回傳供前端渲染的特殊資訊。
pub fn apply(state: &mut State, mv: Move) -> ApplyInfo {
    let (fc, fr) = mv.from;
    let (tc, tr) = mv.to;
    let mut piece = state.board[fr as usize][fc as usize].take().expect("from empty");
    let me = piece.color;
    let captured = state.board[tr as usize][tc as usize].is_some();
    let mut info = ApplyInfo::default();

    // 過路兵吃子：目標為 ep 空格且為兵
    let mut ep_capture = false;
    if piece.kind == PieceKind::Pawn && Some((tc, tr)) == state.ep && !captured {
        let cap_r = fr; // 被吃兵與行棋兵同列、在目標格的後方
        state.board[cap_r as usize][tc as usize] = None;
        info.ep_capture = Some((tc, cap_r));
        ep_capture = true;
    }

    // 王車易位：移動車
    if piece.kind == PieceKind::King && (tc - fc).abs() == 2 {
        let home_r = fr;
        if tc == 6 {
            let rook = state.board[home_r as usize][7].take();
            state.board[home_r as usize][5] = rook;
            info.castle = Some(Castle::King);
        } else {
            let rook = state.board[home_r as usize][0].take();
            state.board[home_r as usize][3] = rook;
            info.castle = Some(Castle::Queen);
        }
    }

    // 升變
    if let Some(promo) = mv.promo {
        piece.kind = promo;
        info.promo = Some(promo);
    }

    state.board[tr as usize][tc as usize] = Some(piece);

    // 易位權更新：王/車離開或車被吃
    match (me, piece.kind) {
        (Color::White, PieceKind::King) => {
            state.castling[0] = false;
            state.castling[1] = false;
        }
        (Color::Black, PieceKind::King) => {
            state.castling[2] = false;
            state.castling[3] = false;
        }
        _ => {}
    }
    revoke_rook_rights(state, (fc, fr));
    revoke_rook_rights(state, (tc, tr)); // 車被吃也撤權

    // 過路兵目標
    state.ep = if piece.kind == PieceKind::Pawn && (tr - fr).abs() == 2 {
        Some((fc, (fr + tr) / 2))
    } else {
        None
    };

    // 50 步計數
    if piece.kind == PieceKind::Pawn || captured || ep_capture {
        state.halfmove = 0;
    } else {
        state.halfmove = state.halfmove.saturating_add(1);
    }

    state.turn = me.opponent();
    info
}

fn revoke_rook_rights(state: &mut State, sq: (i8, i8)) {
    match sq {
        (0, 0) => state.castling[1] = false,
        (7, 0) => state.castling[0] = false,
        (0, 7) => state.castling[3] = false,
        (7, 7) => state.castling[2] = false,
        _ => {}
    }
}

/// 全合法走步（已過濾送將）。
pub fn legal_moves(state: &State) -> Vec<Move> {
    let me = state.turn;
    pseudo_moves(state)
        .into_iter()
        .filter(|&mv| {
            let mut next = state.clone();
            apply(&mut next, mv);
            !is_in_check(&next, me)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Checkmate { winner: Color },
    Stalemate,
    Draw, // 50 步 / 子力不足
}

fn insufficient_material(state: &State) -> bool {
    let mut minors = 0;
    for r in 0..8 {
        for c in 0..8 {
            if let Some(p) = state.board[r][c] {
                match p.kind {
                    PieceKind::King => {}
                    PieceKind::Knight | PieceKind::Bishop => minors += 1,
                    _ => return false, // 有兵/車/后 → 仍可將死
                }
            }
        }
    }
    minors <= 1
}

pub fn status(state: &State) -> Outcome {
    if legal_moves(state).is_empty() {
        return if is_in_check(state, state.turn) {
            Outcome::Checkmate { winner: state.turn.opponent() }
        } else {
            Outcome::Stalemate
        };
    }
    if state.halfmove >= N_HALFMOVE_DRAW || insufficient_material(state) {
        return Outcome::Draw;
    }
    Outcome::Continue
}

#[cfg(test)]
mod tests;

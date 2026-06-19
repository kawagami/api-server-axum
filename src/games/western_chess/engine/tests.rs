use super::*;
use crate::games::western_chess::types::{Color, Move, Piece, PieceKind, State};

fn empty() -> State {
    State {
        board: [[None; 8]; 8],
        turn: Color::White,
        castling: [false; 4],
        ep: None,
        halfmove: 0,
    }
}

fn put(s: &mut State, c: i8, r: i8, kind: PieceKind, color: Color) {
    s.board[r as usize][c as usize] = Some(Piece { kind, color });
}

fn m(from: (i8, i8), to: (i8, i8)) -> Move {
    Move { from, to, promo: None }
}

fn has_move(s: &State, from: (i8, i8), to: (i8, i8)) -> bool {
    legal_moves(s).iter().any(|mv| mv.from == from && mv.to == to)
}

#[test]
fn initial_has_32_pieces_white_first() {
    let s = initial_state();
    let n = s.board.iter().flatten().filter(|p| p.is_some()).count();
    assert_eq!(n, 32);
    assert_eq!(s.turn, Color::White);
}

#[test]
fn pawn_double_then_en_passant_capture() {
    let mut s = empty();
    put(&mut s, 4, 4, PieceKind::Pawn, Color::White); // e5
    put(&mut s, 3, 6, PieceKind::Pawn, Color::Black); // d7
    s.turn = Color::Black;
    apply(&mut s, m((3, 6), (3, 4))); // d7-d5 (double)
    assert_eq!(s.ep, Some((3, 5))); // ep target d6
    // 白 e5 過路吃 d6
    let info = apply(&mut s, m((4, 4), (3, 5)));
    assert_eq!(info.ep_capture, Some((3, 4)));
    assert!(s.board[4][3].is_none()); // 被吃黑兵移除
    assert!(s.board[5][3].is_some()); // 白兵到 d6
}

#[test]
fn castling_kingside_moves_rook() {
    let mut s = empty();
    put(&mut s, 4, 0, PieceKind::King, Color::White);
    put(&mut s, 7, 0, PieceKind::Rook, Color::White);
    put(&mut s, 4, 7, PieceKind::King, Color::Black);
    s.castling[0] = true; // WK
    assert!(has_move(&s, (4, 0), (6, 0)));
    let info = apply(&mut s, m((4, 0), (6, 0)));
    assert_eq!(info.castle, Some(Castle::King));
    assert_eq!(s.board[0][6].unwrap().kind, PieceKind::King);
    assert_eq!(s.board[0][5].unwrap().kind, PieceKind::Rook);
}

#[test]
fn cannot_castle_through_attacked_square() {
    let mut s = empty();
    put(&mut s, 4, 0, PieceKind::King, Color::White);
    put(&mut s, 7, 0, PieceKind::Rook, Color::White);
    put(&mut s, 4, 7, PieceKind::King, Color::Black);
    put(&mut s, 5, 7, PieceKind::Rook, Color::Black); // 黑車控 f 線，攻 f1（王經過格）
    s.castling[0] = true;
    assert!(!has_move(&s, (4, 0), (6, 0)));
}

#[test]
fn pawn_promotion_to_queen() {
    let mut s = empty();
    put(&mut s, 0, 6, PieceKind::Pawn, Color::White); // a7
    put(&mut s, 4, 0, PieceKind::King, Color::White);
    put(&mut s, 4, 7, PieceKind::King, Color::Black);
    // 升變走步應生成（含 4 種）
    let promos: Vec<_> = legal_moves(&s)
        .into_iter()
        .filter(|mv| mv.from == (0, 6) && mv.to == (0, 7))
        .collect();
    assert_eq!(promos.len(), 4);
    apply(&mut s, Move { from: (0, 6), to: (0, 7), promo: Some(PieceKind::Queen) });
    assert_eq!(s.board[7][0].unwrap().kind, PieceKind::Queen);
}

#[test]
fn fools_mate_is_checkmate() {
    let mut s = initial_state();
    apply(&mut s, m((5, 1), (5, 2))); // f3
    apply(&mut s, m((4, 6), (4, 4))); // e5
    apply(&mut s, m((6, 1), (6, 3))); // g4
    apply(&mut s, m((3, 7), (7, 3))); // Qh4#
    assert!(is_in_check(&s, Color::White));
    assert_eq!(status(&s), Outcome::Checkmate { winner: Color::Black });
}

#[test]
fn stalemate_detected() {
    let mut s = empty();
    put(&mut s, 0, 7, PieceKind::King, Color::Black); // a8
    put(&mut s, 0, 5, PieceKind::King, Color::White); // a6
    put(&mut s, 1, 5, PieceKind::Queen, Color::White); // b6
    s.turn = Color::Black;
    assert!(!is_in_check(&s, Color::Black));
    assert!(legal_moves(&s).is_empty());
    assert_eq!(status(&s), Outcome::Stalemate);
}

#[test]
fn fifty_move_rule_draws() {
    let mut s = initial_state();
    s.halfmove = 100;
    assert_eq!(status(&s), Outcome::Draw);
}

#[test]
fn insufficient_material_draws() {
    let mut s = empty();
    put(&mut s, 4, 0, PieceKind::King, Color::White);
    put(&mut s, 4, 7, PieceKind::King, Color::Black);
    assert_eq!(status(&s), Outcome::Draw);
}

#[test]
fn cannot_leave_king_in_check() {
    let mut s = empty();
    put(&mut s, 4, 0, PieceKind::King, Color::White); // e1
    put(&mut s, 3, 1, PieceKind::Bishop, Color::White); // d2 釘住
    put(&mut s, 0, 4, PieceKind::Bishop, Color::Black); // a5 沿 a5-e1 斜線盯王
    put(&mut s, 4, 7, PieceKind::King, Color::Black);
    // d2 象移開會送將 → 不在合法步
    assert!(!has_move(&s, (3, 1), (4, 2)));
}

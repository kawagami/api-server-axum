use super::*;
use crate::games::chess::types::{GameState, Move, PieceKind, Side, Square};

fn empty_board() -> Board {
    Default::default()
}

fn put(board: &mut Board, col: i8, row: i8, kind: PieceKind, side: Side) {
    board[row as usize][col as usize] = Some(Piece::new(kind, side));
}

fn sq(col: i8, row: i8) -> Square {
    Square::new(col, row)
}

fn mv(fc: i8, fr: i8, tc: i8, tr: i8) -> Move {
    Move {
        from: sq(fc, fr),
        to: sq(tc, tr),
    }
}

/// 只有兩個將的最小盤（不對臉），避免無關干擾。
fn bare_kings(turn: Side) -> GameState {
    let mut board = empty_board();
    put(&mut board, 4, 0, PieceKind::General, Side::Red);
    put(&mut board, 3, 9, PieceKind::General, Side::Black);
    GameState {
        board,
        turn,
        halfmove_no_capture: 0,
    }
}

#[test]
fn initial_position_has_32_pieces() {
    let s = initial_state();
    let count = s.board.iter().flatten().filter(|p| p.is_some()).count();
    assert_eq!(count, 32);
    assert_eq!(s.turn, Side::Red);
}

#[test]
fn rook_moves_straight_and_blocks() {
    let mut s = bare_kings(Side::Red);
    put(&mut s.board, 0, 0, PieceKind::Rook, Side::Red);
    // 直走到空格 OK
    assert!(is_legal(&s, mv(0, 0, 0, 5)).is_ok());
    // 斜走非法
    assert_eq!(is_legal(&s, mv(0, 0, 1, 1)), Err(IllegalReason::BadMove));
    // 路徑有子 → 阻擋
    put(&mut s.board, 0, 3, PieceKind::Soldier, Side::Red);
    assert_eq!(is_legal(&s, mv(0, 0, 0, 5)), Err(IllegalReason::BlockedPath));
}

#[test]
fn horse_leg_blocked() {
    let mut s = bare_kings(Side::Red);
    put(&mut s.board, 1, 0, PieceKind::Horse, Side::Red);
    assert!(is_legal(&s, mv(1, 0, 2, 2)).is_ok());
    // 蹩馬腳：(1,1) 有子擋住直走方向
    put(&mut s.board, 1, 1, PieceKind::Soldier, Side::Red);
    assert_eq!(is_legal(&s, mv(1, 0, 2, 2)), Err(IllegalReason::BadHorseLeg));
}

#[test]
fn cannon_needs_screen_to_capture() {
    let mut s = bare_kings(Side::Red);
    put(&mut s.board, 4, 1, PieceKind::Cannon, Side::Red);
    // 無炮架不能吃黑將（路徑空只能移動到空格）
    // 黑將在 (3,9)，移到同列須 col 一致；放敵子在 (4,8)
    put(&mut s.board, 4, 8, PieceKind::Soldier, Side::Black);
    // 直接吃 (4,8)：中間無炮架 → 非法
    assert_eq!(is_legal(&s, mv(4, 1, 4, 8)), Err(IllegalReason::BlockedPath));
    // 加炮架 (4,4)
    put(&mut s.board, 4, 4, PieceKind::Soldier, Side::Black);
    assert!(is_legal(&s, mv(4, 1, 4, 8)).is_ok());
    // 移到空格須路徑全空：(4,4)(4,8) 擋住到 (4,9) 不行；移到 (4,2) 空且無阻 OK
    assert!(is_legal(&s, mv(4, 1, 4, 2)).is_ok());
}

#[test]
fn elephant_cannot_cross_river_and_eye() {
    let mut s = bare_kings(Side::Red);
    put(&mut s.board, 2, 0, PieceKind::Elephant, Side::Red);
    assert!(is_legal(&s, mv(2, 0, 4, 2)).is_ok());
    // 過河：紅象到 row5 之上 → 由 (4,4) 走 (2,6) 跨河
    put(&mut s.board, 4, 4, PieceKind::Elephant, Side::Red);
    s.board[0][2] = None;
    assert_eq!(is_legal(&s, mv(4, 4, 2, 6)), Err(IllegalReason::CrossRiver));
    // 塞象眼
    let mut s2 = bare_kings(Side::Red);
    put(&mut s2.board, 2, 0, PieceKind::Elephant, Side::Red);
    put(&mut s2.board, 3, 1, PieceKind::Soldier, Side::Red);
    assert_eq!(
        is_legal(&s2, mv(2, 0, 4, 2)),
        Err(IllegalReason::BadElephantEye)
    );
}

#[test]
fn advisor_and_general_confined_to_palace() {
    let mut s = bare_kings(Side::Red);
    put(&mut s.board, 3, 0, PieceKind::Advisor, Side::Red);
    assert!(is_legal(&s, mv(3, 0, 4, 1)).is_ok());
    assert_eq!(is_legal(&s, mv(3, 0, 2, 1)), Err(IllegalReason::OutOfPalace));
    // 將橫走出宮
    assert_eq!(is_legal(&s, mv(4, 0, 4, 3)), Err(IllegalReason::BadMove)); // 一次走兩格
    assert!(is_legal(&s, mv(4, 0, 4, 1)).is_ok());
}

#[test]
fn soldier_forward_then_sideways_after_river() {
    let mut s = bare_kings(Side::Red);
    put(&mut s.board, 0, 3, PieceKind::Soldier, Side::Red);
    assert!(is_legal(&s, mv(0, 3, 0, 4)).is_ok());
    // 未過河不能橫走
    assert_eq!(is_legal(&s, mv(0, 3, 1, 3)), Err(IllegalReason::BadMove));
    // 不能後退
    assert_eq!(is_legal(&s, mv(0, 3, 0, 2)), Err(IllegalReason::BadMove));
    // 過河後可橫走
    let mut s2 = bare_kings(Side::Red);
    put(&mut s2.board, 0, 5, PieceKind::Soldier, Side::Red);
    assert!(is_legal(&s2, mv(0, 5, 1, 5)).is_ok());
}

#[test]
fn flying_general_move_is_illegal() {
    // 兩將同 col 4，中間僅一兵；移開該兵造成對臉 → FlyingGeneral
    let mut board = empty_board();
    put(&mut board, 4, 0, PieceKind::General, Side::Red);
    put(&mut board, 4, 9, PieceKind::General, Side::Black);
    put(&mut board, 4, 4, PieceKind::Soldier, Side::Red);
    let s = GameState {
        board,
        turn: Side::Red,
        halfmove_no_capture: 0,
    };
    // 兵橫走不行（未過河），改用過河兵測：放過河紅兵在 (4,5)
    let mut board2 = empty_board();
    put(&mut board2, 4, 0, PieceKind::General, Side::Red);
    put(&mut board2, 4, 9, PieceKind::General, Side::Black);
    put(&mut board2, 4, 5, PieceKind::Soldier, Side::Red);
    let s2 = GameState {
        board: board2,
        turn: Side::Red,
        halfmove_no_capture: 0,
    };
    // 兵從 (4,5) 橫走到 (3,5) → 中間清空 → 兩將對臉 → FlyingGeneral
    assert_eq!(
        is_legal(&s2, mv(4, 5, 3, 5)),
        Err(IllegalReason::FlyingGeneral)
    );
    let _ = s;
}

#[test]
fn cannot_leave_own_general_in_check() {
    // 黑車盯紅將同列，紅士擋在中間；士移開致送將 → LeavesKingInCheck
    let mut board = empty_board();
    put(&mut board, 4, 0, PieceKind::General, Side::Red);
    put(&mut board, 0, 9, PieceKind::General, Side::Black); // 避開飛將線
    put(&mut board, 4, 5, PieceKind::Rook, Side::Black); // 黑車 col4 盯紅將
    put(&mut board, 4, 1, PieceKind::Advisor, Side::Red); // 擋在中間
    let s = GameState {
        board,
        turn: Side::Red,
        halfmove_no_capture: 0,
    };
    // 現在沒被將（士擋著）
    assert!(!is_in_check(&s, Side::Red));
    // 士斜走離開 col4 → 紅將被車將 → LeavesKingInCheck
    assert_eq!(
        is_legal(&s, mv(4, 1, 3, 2)),
        Err(IllegalReason::LeavesKingInCheck)
    );
}

#[test]
fn checkmate_detected() {
    // 黑將 (4,9) 被紅車 (4,7) 沿 col4 將軍；兩側紅車封 col3、col5 退路。
    let mut board = empty_board();
    put(&mut board, 0, 0, PieceKind::General, Side::Red);
    put(&mut board, 4, 9, PieceKind::General, Side::Black);
    put(&mut board, 4, 7, PieceKind::Rook, Side::Red); // 沿 col4 將軍
    put(&mut board, 3, 7, PieceKind::Rook, Side::Red); // 封 col3 → (3,9) 不可
    put(&mut board, 5, 7, PieceKind::Rook, Side::Red); // 封 col5 → (5,9) 不可
    let s = GameState {
        board,
        turn: Side::Black,
        halfmove_no_capture: 0,
    };
    assert!(is_in_check(&s, Side::Black));
    assert_eq!(game_status(&s), Status::Checkmate { winner: Side::Red });
}

#[test]
fn stalemate_is_a_loss() {
    // 行棋方無合法步且未被將 → 困斃判負。
    // 黑將 (3,9) 角落，紅車封 row9 與 col4，黑將無處可走且未被將。
    let mut board = empty_board();
    put(&mut board, 0, 0, PieceKind::General, Side::Red);
    put(&mut board, 3, 9, PieceKind::General, Side::Black);
    // 黑將可走 (4,9),(3,8) → 封住。紅車 col4 (4,0) 封 (4,9)；紅車 row8 (0,8) 封 (3,8)
    put(&mut board, 4, 0, PieceKind::Rook, Side::Red); // 控 col4 → (4,9) 不可
    put(&mut board, 0, 8, PieceKind::Rook, Side::Red); // 控 row8 → (3,8) 不可
    let s = GameState {
        board,
        turn: Side::Black,
        halfmove_no_capture: 0,
    };
    assert!(!is_in_check(&s, Side::Black));
    assert_eq!(game_status(&s), Status::Stalemate { loser: Side::Black });
}

#[test]
fn sixty_move_rule_draws() {
    let mut s = bare_kings(Side::Red);
    s.halfmove_no_capture = 120;
    // 確保仍有合法步（將可動）
    assert!(!matches!(game_status(&s), Status::Checkmate { .. }));
    assert_eq!(game_status(&s), Status::Draw);
}

#[test]
fn apply_resets_counter_on_capture() {
    let mut s = bare_kings(Side::Red);
    put(&mut s.board, 0, 0, PieceKind::Rook, Side::Red);
    put(&mut s.board, 0, 5, PieceKind::Soldier, Side::Black);
    s.halfmove_no_capture = 10;
    apply(&mut s, mv(0, 0, 0, 5)); // 吃黑兵
    assert_eq!(s.halfmove_no_capture, 0);
    assert_eq!(s.turn, Side::Black);
    // 非吃子 +1
    let mut s2 = bare_kings(Side::Red);
    s2.halfmove_no_capture = 10;
    apply(&mut s2, mv(4, 0, 4, 1));
    assert_eq!(s2.halfmove_no_capture, 11);
}

#[test]
fn legal_moves_filters_self_check() {
    let s = initial_state();
    // 開局紅炮 (1,2) 應有多個合法目標
    let moves = legal_moves(&s, sq(1, 2));
    assert!(!moves.is_empty());
    // 非行棋方（黑子）回空
    assert!(legal_moves(&s, sq(0, 9)).is_empty());
}

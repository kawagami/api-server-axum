use super::*;

fn empty(first_color: Color, turn: Side) -> BanqiState {
    BanqiState {
        board: [[Cell::Empty; 8]; 4],
        turn,
        first_color: Some(first_color),
        quiet: 0,
    }
}

fn up(s: &mut BanqiState, col: i8, row: i8, color: Color, kind: Kind) {
    s.board[row as usize][col as usize] = Cell::Up(Piece { color, kind });
}

fn hidden(s: &mut BanqiState, col: i8, row: i8, color: Color, kind: Kind) {
    s.board[row as usize][col as usize] = Cell::Hidden(Piece { color, kind });
}

#[test]
fn first_flip_assigns_color() {
    let mut deck = vec![Piece { color: Color::Black, kind: Kind::Horse }; 32];
    deck[0] = Piece { color: Color::Red, kind: Kind::King }; // (col0,row0)
    let mut s = from_deck(deck);
    assert!(s.first_color.is_none());
    let eff = apply_action(&mut s, Side::First, Action::Flip { col: 0, row: 0 }).unwrap();
    assert_eq!(eff, Effect::Flipped { col: 0, row: 0, piece: Piece { color: Color::Red, kind: Kind::King } });
    assert_eq!(s.first_color, Some(Color::Red)); // First 翻到紅 → First 執紅
    assert_eq!(s.turn, Side::Second); // 換手
}

#[test]
fn second_flips_first_gets_opposite() {
    let mut deck = vec![Piece { color: Color::Black, kind: Kind::Horse }; 32];
    deck[0] = Piece { color: Color::Red, kind: Kind::King };
    let mut s = from_deck(deck);
    s.turn = Side::Second; // 假設 Second 先翻
    apply_action(&mut s, Side::Second, Action::Flip { col: 0, row: 0 }).unwrap();
    // Second 翻到紅 → Second 執紅 → First 執黑
    assert_eq!(s.first_color, Some(Color::Black));
}

#[test]
fn move_before_color_assigned_rejected() {
    let deck = vec![Piece { color: Color::Red, kind: Kind::Pawn }; 32];
    let mut s = from_deck(deck);
    let r = apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (1, 0) });
    assert_eq!(r, Err("no_color_yet"));
}

#[test]
fn pawn_captures_king_but_not_reverse() {
    // First 執紅
    let mut s = empty(Color::Red, Side::First);
    up(&mut s, 0, 0, Color::Red, Kind::Pawn);
    up(&mut s, 1, 0, Color::Black, Kind::King);
    let eff = apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (1, 0) }).unwrap();
    assert_eq!(
        eff,
        Effect::Moved { from: (0, 0), to: (1, 0), captured: Some(Piece { color: Color::Black, kind: Kind::King }) }
    );

    let mut s2 = empty(Color::Red, Side::First);
    up(&mut s2, 0, 0, Color::Red, Kind::King);
    up(&mut s2, 1, 0, Color::Black, Kind::Pawn);
    assert_eq!(
        apply_action(&mut s2, Side::First, Action::Move { from: (0, 0), to: (1, 0) }),
        Err("cannot_capture")
    );
}

#[test]
fn rank_capture_rules() {
    // 車吃馬 OK
    let mut s = empty(Color::Red, Side::First);
    up(&mut s, 0, 0, Color::Red, Kind::Rook);
    up(&mut s, 1, 0, Color::Black, Kind::Horse);
    assert!(apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (1, 0) }).is_ok());
    // 馬吃車 不行
    let mut s2 = empty(Color::Red, Side::First);
    up(&mut s2, 0, 0, Color::Red, Kind::Horse);
    up(&mut s2, 1, 0, Color::Black, Kind::Rook);
    assert_eq!(
        apply_action(&mut s2, Side::First, Action::Move { from: (0, 0), to: (1, 0) }),
        Err("cannot_capture")
    );
}

#[test]
fn non_adjacent_move_rejected() {
    let mut s = empty(Color::Red, Side::First);
    up(&mut s, 0, 0, Color::Red, Kind::Rook);
    assert_eq!(
        apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (2, 0) }),
        Err("not_adjacent")
    );
}

#[test]
fn cannot_move_opponent_piece() {
    let mut s = empty(Color::Red, Side::First); // First 執紅
    up(&mut s, 0, 0, Color::Black, Kind::Rook); // 黑子
    up(&mut s, 1, 0, Color::Red, Kind::Pawn);
    assert_eq!(
        apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (1, 0) }),
        Err("not_your_piece")
    );
}

#[test]
fn cannon_jumps_one_screen_to_capture() {
    let mut s = empty(Color::Red, Side::First);
    up(&mut s, 0, 0, Color::Red, Kind::Cannon);
    hidden(&mut s, 1, 0, Color::Black, Kind::Pawn); // 炮架（面朝下也算）
    up(&mut s, 3, 0, Color::Black, Kind::King); // 目標：任意階皆可吃
    let eff = apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (3, 0) }).unwrap();
    assert_eq!(
        eff,
        Effect::Moved { from: (0, 0), to: (3, 0), captured: Some(Piece { color: Color::Black, kind: Kind::King }) }
    );
}

#[test]
fn cannon_without_screen_rejected() {
    let mut s = empty(Color::Red, Side::First);
    up(&mut s, 0, 0, Color::Red, Kind::Cannon);
    up(&mut s, 3, 0, Color::Black, Kind::King); // 中間無架
    assert_eq!(
        apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (3, 0) }),
        Err("cannon_needs_screen")
    );
}

#[test]
fn cannon_steps_to_empty_when_not_capturing() {
    let mut s = empty(Color::Red, Side::First);
    up(&mut s, 0, 0, Color::Red, Kind::Cannon);
    assert!(apply_action(&mut s, Side::First, Action::Move { from: (0, 0), to: (1, 0) }).is_ok());
}

#[test]
fn win_by_elimination() {
    let mut s = empty(Color::Red, Side::First); // First 紅, Second 黑
    up(&mut s, 0, 0, Color::Red, Kind::King); // 場上只剩紅
    assert_eq!(status(&s), Outcome::Win { winner: Side::First, reason: "elimination" });
}

#[test]
fn ongoing_before_first_flip() {
    let deck = vec![Piece { color: Color::Red, kind: Kind::Pawn }; 32];
    let s = from_deck(deck);
    assert_eq!(status(&s), Outcome::Continue);
}

use super::*;

fn st() -> GoState {
    initial_state()
}

fn b(s: &mut GoState, c: i8, r: i8) {
    s.board[r as usize][c as usize] = Some(Stone::Black);
}
fn w(s: &mut GoState, c: i8, r: i8) {
    s.board[r as usize][c as usize] = Some(Stone::White);
}

#[test]
fn place_legal_and_switches_turn() {
    let mut s = st();
    assert!(is_legal(&s, 3, 3).is_ok());
    apply(&mut s, 3, 3);
    assert_eq!(s.board[3][3], Some(Stone::Black));
    assert_eq!(s.turn, Stone::White);
}

#[test]
fn capture_single_stone() {
    let mut s = st();
    // 白子 (1,1) 被黑三面圍，黑下最後一氣 (1,0) 提子
    w(&mut s, 1, 1);
    b(&mut s, 0, 1);
    b(&mut s, 2, 1);
    b(&mut s, 1, 2);
    s.turn = Stone::Black;
    let captured = apply(&mut s, 1, 0);
    assert_eq!(captured, vec![(1, 1)]);
    assert_eq!(s.board[1][1], None);
}

#[test]
fn suicide_is_illegal() {
    let mut s = st();
    // (1,1) 四面皆白，黑下 (1,1) 無氣且不提子 → 自殺
    w(&mut s, 0, 1);
    w(&mut s, 2, 1);
    w(&mut s, 1, 0);
    w(&mut s, 1, 2);
    s.turn = Stone::Black;
    assert_eq!(is_legal(&s, 1, 1), Err(Reason::Suicide));
}

#[test]
fn capturing_move_is_not_suicide() {
    let mut s = st();
    // 黑 (1,1) 自身會無氣，但提走白一子後重獲氣 → 合法
    // 白 (1,1) 已在；黑下 (1,0) 提白，(1,0) 本身靠提子得氣
    w(&mut s, 1, 1);
    b(&mut s, 0, 1);
    b(&mut s, 2, 1);
    b(&mut s, 1, 2);
    w(&mut s, 0, 0);
    w(&mut s, 2, 0);
    s.turn = Stone::Black;
    assert!(is_legal(&s, 1, 0).is_ok());
}

#[test]
fn ko_forbids_immediate_recapture() {
    let mut s = st();
    // 鑽石劫：黑下 P=(1,2) 提白 T=(2,2)，黑成單子單氣 → 設劫點 (2,2)
    b(&mut s, 3, 2);
    b(&mut s, 2, 1);
    b(&mut s, 2, 3);
    w(&mut s, 2, 2); // T
    w(&mut s, 0, 2);
    w(&mut s, 1, 1);
    w(&mut s, 1, 3);
    s.turn = Stone::Black;
    let captured = apply(&mut s, 1, 2); // P
    assert_eq!(captured, vec![(2, 2)]);
    assert_eq!(s.ko, Some((2, 2)));
    // 白立即回提 (2,2) → 劫禁
    assert_eq!(is_legal(&s, 2, 2), Err(Reason::Ko));
}

#[test]
fn two_passes_end_game_white_wins_empty_by_komi() {
    let mut s = st();
    pass(&mut s); // 黑虛手
    assert_eq!(status(&s), Outcome::Continue);
    pass(&mut s); // 白虛手 → 終局
    // 空盤 0:0，白 +7.5 → 白勝
    assert_eq!(status(&s), Outcome::Win(Stone::White));
}

#[test]
fn pass_then_play_resets_counter() {
    let mut s = st();
    pass(&mut s);
    apply(&mut s, 3, 3);
    assert_eq!(s.passes, 0);
}

#[test]
fn area_score_counts_stones_and_territory() {
    let mut s = st();
    // 黑封死角 (0,0)；白子放遠處，使大片空白同鄰兩色 → 中立，只剩 (0,0) 為黑地
    b(&mut s, 0, 1);
    b(&mut s, 1, 0);
    b(&mut s, 1, 1);
    w(&mut s, 10, 10);
    let (black, white) = score(&s.board);
    assert_eq!(black, 4); // 3 子 + 角地 (0,0)
    assert_eq!(white, 1); // 1 子，大片空白為中立不計
}

#[test]
fn lone_color_owns_whole_board() {
    let mut s = st();
    b(&mut s, 3, 3); // 只有黑子 → 全盤皆黑（area scoring）
    let (black, white) = score(&s.board);
    assert_eq!(black, 361);
    assert_eq!(white, 0);
}

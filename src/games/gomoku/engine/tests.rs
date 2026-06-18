use super::*;

fn place_seq(state: &mut GomokuState, moves: &[(i8, i8)]) {
    for &(c, r) in moves {
        assert!(is_legal(state, c, r).is_ok(), "illegal at {c},{r}");
        apply(state, c, r);
    }
}

#[test]
fn initial_is_black_to_move_empty() {
    let s = initial_state();
    assert_eq!(s.turn, Stone::Black);
    assert_eq!(s.placed, 0);
    assert_eq!(status(&s), Outcome::Continue);
}

#[test]
fn reject_out_of_bounds_and_occupied() {
    let mut s = initial_state();
    assert_eq!(is_legal(&s, 15, 0), Err("bad_coord"));
    assert_eq!(is_legal(&s, -1, 0), Err("bad_coord"));
    apply(&mut s, 7, 7);
    assert_eq!(is_legal(&s, 7, 7), Err("occupied"));
}

#[test]
fn horizontal_five_black_wins() {
    let mut s = initial_state();
    // 黑 (0..5, 0)，白穿插別處
    place_seq(
        &mut s,
        &[
            (0, 0),
            (0, 5),
            (1, 0),
            (1, 5),
            (2, 0),
            (2, 5),
            (3, 0),
            (3, 5),
            (4, 0), // 黑第五子
        ],
    );
    assert_eq!(status(&s), Outcome::Win(Stone::Black));
}

#[test]
fn diagonal_five_wins() {
    let mut s = initial_state();
    place_seq(
        &mut s,
        &[
            (0, 0),
            (0, 1),
            (1, 1),
            (0, 2),
            (2, 2),
            (0, 3),
            (3, 3),
            (0, 4),
            (4, 4), // 黑對角第五
        ],
    );
    assert_eq!(status(&s), Outcome::Win(Stone::Black));
}

#[test]
fn four_in_row_not_win() {
    let mut s = initial_state();
    place_seq(&mut s, &[(0, 0), (0, 5), (1, 0), (1, 5), (2, 0), (2, 5), (3, 0)]);
    assert_eq!(status(&s), Outcome::Continue);
}

#[test]
fn overline_six_still_wins() {
    let mut s = initial_state();
    // 自由規則：6 連也算勝。白子打散在 col7（隔列，不成線）
    place_seq(
        &mut s,
        &[
            (0, 0),
            (7, 0),
            (1, 0),
            (7, 2),
            (2, 0),
            (7, 4),
            (3, 0),
            (7, 6),
            (5, 0), // 黑第五子（此時 0..3 + 5，未連五）
            (7, 8),
            (4, 0), // 黑第六子補中間 → col 0..6 row0 六連
        ],
    );
    assert_eq!(status(&s), Outcome::Win(Stone::Black));
}

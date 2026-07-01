use super::*;
use crate::games::avalon::roles::{build_roles, good_evil, known_seats, Alignment, Options, Role};

// 固定 5 人局：座位 0..4。
// 0 Merlin, 1 Percival, 2 LoyalServant (好) ; 3 Assassin, 4 Morgana (壞)
fn five() -> AvalonState {
    setup_with_roles(
        vec![
            Role::Merlin,
            Role::Percival,
            Role::LoyalServant,
            Role::Assassin,
            Role::Morgana,
        ],
        0,
    )
}

#[test]
fn role_counts_per_size() {
    assert_eq!(good_evil(5), Some((3, 2)));
    assert_eq!(good_evil(10), Some((6, 4)));
    let r = build_roles(7, Options { mordred: true, oberon: false }).unwrap();
    assert_eq!(r.len(), 7);
    assert_eq!(r.iter().filter(|x| x.alignment() == Alignment::Evil).count(), 3);
}

#[test]
fn too_many_special_evil_rejected() {
    // 5 人只有 2 壞，要 Assassin+Morgana+Mordred+Oberon = 4 → 爆
    let e = build_roles(5, Options { mordred: true, oberon: true });
    assert_eq!(e, Err("too_many_special_evil"));
}

#[test]
fn merlin_sees_evil_percival_sees_merlin_morgana() {
    let s = five();
    // 梅林(0) 看到壞人 3,4
    assert_eq!(known_seats(&s.roles, 0), vec![3, 4]);
    // 派西維爾(1) 看到梅林0 與莫甘娜4
    assert_eq!(known_seats(&s.roles, 1), vec![0, 4]);
    // 忠臣(2) 無資訊
    assert!(known_seats(&s.roles, 2).is_empty());
}

#[test]
fn mordred_hidden_from_merlin() {
    let s = setup_with_roles(
        vec![Role::Merlin, Role::LoyalServant, Role::Percival, Role::Assassin, Role::Mordred, Role::Minion, Role::LoyalServant],
        0,
    );
    let seen = known_seats(&s.roles, 0); // 梅林
    assert!(seen.contains(&3)); // Assassin 看得到
    assert!(!seen.contains(&4)); // Mordred 看不到
}

#[test]
fn propose_validates_leader_and_size() {
    let mut s = five();
    assert_eq!(propose_team(&mut s, 1, &[0, 1]), Err("not_leader"));
    assert_eq!(propose_team(&mut s, 0, &[0]), Err("bad_team_size")); // round0 需 2 人
    assert_eq!(propose_team(&mut s, 0, &[0, 0]), Err("bad_team")); // 重複
    assert!(propose_team(&mut s, 0, &[0, 1]).is_ok());
    assert_eq!(s.phase, Phase::TeamVote);
}

#[test]
fn vote_approve_enters_quest() {
    let mut s = five();
    propose_team(&mut s, 0, &[0, 1]).unwrap();
    for seat in 0..4 {
        assert!(team_vote(&mut s, seat, true).unwrap().is_none());
    }
    let tally = team_vote(&mut s, 4, true).unwrap().unwrap();
    assert!(tally.approved);
    assert_eq!(s.phase, Phase::Quest);
}

#[test]
fn vote_reject_advances_leader() {
    let mut s = five();
    propose_team(&mut s, 0, &[0, 1]).unwrap();
    for seat in 0..4 {
        team_vote(&mut s, seat, false).unwrap();
    }
    let tally = team_vote(&mut s, 4, false).unwrap().unwrap();
    assert!(!tally.approved);
    assert_eq!(s.leader, 1);
    assert_eq!(s.phase, Phase::TeamBuilding);
    assert_eq!(s.rejects, 1);
}

#[test]
fn five_rejects_evil_wins() {
    let mut s = five();
    for _ in 0..5 {
        let l = s.leader;
        propose_team(&mut s, l, &[0, 1]).unwrap();
        for seat in 0..s.n {
            team_vote(&mut s, seat, false).unwrap();
        }
    }
    assert_eq!(s.phase, Phase::GameOver);
    assert_eq!(s.winner, Some(Alignment::Evil));
    assert_eq!(s.reason, "evil_five_rejects");
}

#[test]
fn good_cannot_fail_quest() {
    let mut s = five();
    propose_team(&mut s, 0, &[0, 1]).unwrap(); // 0 梅林、1 派西(都好)
    for seat in 0..s.n {
        team_vote(&mut s, seat, true).unwrap();
    }
    assert_eq!(quest_card(&mut s, 0, false), Err("good_must_succeed"));
    assert_eq!(quest_card(&mut s, 2, true), Err("not_on_team")); // 2 不在隊
}

#[test]
fn one_fail_fails_quest_round0() {
    let mut s = five();
    propose_team(&mut s, 0, &[1, 3]).unwrap(); // 含壞人 3
    for seat in 0..s.n {
        team_vote(&mut s, seat, true).unwrap();
    }
    quest_card(&mut s, 1, true).unwrap();
    let t = quest_card(&mut s, 3, false).unwrap().unwrap();
    assert_eq!(t.fails, 1);
    assert!(!t.success);
    assert_eq!(s.results, vec![false]);
}

#[test]
fn three_successes_then_assassin_decides() {
    let mut s = five();
    // 直接灌 3 次成功任務結果，模擬到刺客階段
    s.results = vec![true, true];
    s.round = 2;
    let l = s.leader;
    propose_team(&mut s, l, &[0, 1]).unwrap();
    for seat in 0..s.n {
        team_vote(&mut s, seat, true).unwrap();
    }
    quest_card(&mut s, 0, true).unwrap();
    quest_card(&mut s, 1, true).unwrap();
    assert_eq!(s.phase, Phase::Assassinate);
    // 刺客(3) 猜中梅林(0) → 壞人逆轉
    let r = assassinate(&mut s, 3, 0).unwrap();
    assert!(r.correct);
    assert_eq!(s.winner, Some(Alignment::Evil));
    assert_eq!(s.reason, "evil_assassinate");
}

#[test]
fn assassin_miss_good_wins() {
    let mut s = five();
    s.results = vec![true, true, true];
    s.phase = Phase::Assassinate;
    let r = assassinate(&mut s, 3, 2).unwrap(); // 猜忠臣，非梅林
    assert!(!r.correct);
    assert_eq!(s.winner, Some(Alignment::Good));
    assert_eq!(s.reason, "good_assassin_miss");
}

#[test]
fn fails_required_two_on_fourth_quest_big_table() {
    assert_eq!(roles::fails_required(7, 3), 2);
    assert_eq!(roles::fails_required(5, 3), 1);
    assert_eq!(roles::fails_required(7, 0), 1);
}

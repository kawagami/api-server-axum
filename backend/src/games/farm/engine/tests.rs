use super::*;

#[test]
fn initial_state_shape() {
    let gs = initial_state(3);
    assert_eq!(gs.players.len(), 3);
    assert_eq!(gs.round, 1);
    assert_eq!(gs.phase, Phase::Placing);
    let f = &gs.players[0];
    assert_eq!(f.rooms, 2);
    assert_eq!(f.family, 2);
    assert_eq!(f.used_tiles(), 2);
    assert_eq!(f.free_tiles(), 13);
}

#[test]
fn plow_then_sow_grain() {
    let mut f = Farm::new();
    apply_action(&mut f, Action::Plow, 0, Input::default()).unwrap();
    assert_eq!(f.fields.len(), 1);
    f.grain = 1;
    apply_action(&mut f, Action::Sow, 0, Input { grain_fields: 1, ..Default::default() }).unwrap();
    assert_eq!(f.fields[0].crop, Some((Crop::Grain, 3)));
    assert_eq!(f.grain, 0);
}

#[test]
fn capacity_blocks_overbuild() {
    let mut f = Farm::new();
    for _ in 0..13 {
        apply_action(&mut f, Action::Plow, 0, Input::default()).unwrap();
    }
    assert_eq!(f.free_tiles(), 0);
    assert_eq!(apply_action(&mut f, Action::Plow, 0, Input::default()), Err("no_space"));
}

#[test]
fn build_rooms_then_family_growth() {
    let mut f = Farm::new();
    f.wood = 5;
    f.reed = 2;
    apply_action(&mut f, Action::BuildRooms, 0, Input { rooms: 1, ..Default::default() }).unwrap();
    assert_eq!(f.rooms, 3);
    apply_action(&mut f, Action::FamilyGrowth, 0, Input::default()).unwrap();
    assert_eq!(f.family, 3);
    // 房間用滿則不可再成長
    assert_eq!(apply_action(&mut f, Action::FamilyGrowth, 0, Input::default()), Err("no_room"));
}

#[test]
fn harvest_field_yields_to_supply() {
    let mut f = Farm::new();
    f.family = 0; // 免餵食干擾
    f.fields.push(Field { crop: Some((Crop::Grain, 3)) });
    harvest(&mut f);
    assert_eq!(f.grain, 1);
    assert_eq!(f.fields[0].crop, Some((Crop::Grain, 2)));
}

#[test]
fn feeding_shortfall_creates_begging() {
    let mut f = Farm::new(); // family 2 → 需 4 糧，無任何食物
    harvest(&mut f);
    assert_eq!(f.begging, 4);
    assert_eq!(f.food, 0);
}

#[test]
fn feeding_converts_grain_then_begs() {
    let mut f = Farm::new();
    f.family = 1; // 需 2
    f.grain = 1;
    harvest(&mut f);
    assert_eq!(f.grain, 0); // 1 穀煮成 1 糧
    assert_eq!(f.begging, 1); // 仍缺 1
}

#[test]
fn animals_breed_when_pair_and_space() {
    let mut f = Farm::new();
    f.family = 1;
    f.food = 2; // 餵飽，不動到羊
    f.pastures.push(Pasture { tiles: 2, stable: false, animal: None }); // 容量足
    f.sheep = 2;
    harvest(&mut f);
    assert_eq!(f.sheep, 3);
    assert_eq!(f.begging, 0);
}

#[test]
fn empty_farm_score() {
    // 7 類各 -1（=-7）+ 家庭 2×3（+6）+ 空格 13（-13）= -14
    assert_eq!(score(&Farm::new()), -14);
}

#[test]
fn turn_order_cycles_and_round_advances() {
    let mut gs = initial_state(2);
    assert_eq!(current_player(&gs), Some(0));
    take_action(&mut gs, 0, Action::Plow, Input::default()).unwrap();
    assert_eq!(current_player(&gs), Some(1));
    take_action(&mut gs, 1, Action::GrainSeeds, Input::default()).unwrap();
    assert_eq!(current_player(&gs), Some(0));
    take_action(&mut gs, 0, Action::DayLabor, Input::default()).unwrap();
    take_action(&mut gs, 1, Action::River, Input::default()).unwrap();
    // 4 工人放完 → 進第 2 輪
    assert_eq!(gs.round, 2);
    assert_eq!(current_player(&gs), Some(0));
}

#[test]
fn cannot_take_occupied_or_out_of_turn() {
    let mut gs = initial_state(2);
    take_action(&mut gs, 0, Action::Plow, Input::default()).unwrap();
    // 換 1 行棋，0 再下 → 非其回合
    assert_eq!(take_action(&mut gs, 0, Action::GrainSeeds, Input::default()), Err("not_your_turn"));
    // 1 取已被佔的 Plow
    assert_eq!(take_action(&mut gs, 1, Action::Plow, Input::default()), Err("occupied"));
}

#[test]
fn accumulation_piles_and_resets() {
    let mut gs = initial_state(2);
    // 第 1 輪 Forest 已累積 3 木；玩家 0 取走
    take_action(&mut gs, 0, Action::Forest, Input::default()).unwrap();
    assert_eq!(gs.players[0].wood, 3);
}

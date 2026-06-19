//! 農場經營（家庭版 worker-placement）純函式引擎 — 零 WS 依賴，可單測。
//!
//! 機制原創重做、術語自訂，不含任何商業桌遊素材/文案/名稱。
//!
//! ## Phase-1 範圍與簡化（刻意）
//! - **無職業 / 改良牌組**（家庭版定義）。
//! - **柵欄幾何抽象**：牧場記為「一塊 N 格」+ 是否含畜舍，不做精確圍欄拼塊。
//! - **允許基本煮食轉換**：收穫餵食不足時可把 穀/菜→1 糧、牲畜→2 糧 補足，避免無改良卡時必乞討。
//! - **行動揭示**用可調資料表 `ROUND_ACTIONS`，非官方精確排程。
//! - 農場容量固定 15 格（房 + 田 + 牧場格 + 無圈畜舍 ≤ 15）。

pub const FARM_TILES: u8 = 15;
pub const TOTAL_ROUNDS: u8 = 14;
/// 收穫輪（每階段末）。
pub const HARVEST_ROUNDS: [u8; 6] = [4, 7, 9, 11, 13, 14];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crop {
    Grain,
    Vegetable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Animal {
    Sheep,
    Boar,
    Cattle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum House {
    Wood,
    Clay,
    Stone,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Field {
    /// 田上作物與剩餘數量（None = 已犁未播）。
    pub crop: Option<(CropKind, u8)>,
}

pub type CropKind = Crop;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pasture {
    pub tiles: u8,
    pub stable: bool,
    pub animal: Option<(Animal, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Farm {
    pub house: House,
    pub rooms: u8,
    pub family: u8,
    pub fields: Vec<Field>,
    pub pastures: Vec<Pasture>,
    pub loose_stables: u8, // 未圈入牧場的畜舍
    // 資源
    pub wood: u32,
    pub clay: u32,
    pub reed: u32,
    pub stone: u32,
    pub grain: u32,
    pub veg: u32,
    pub sheep: u32,
    pub boar: u32,
    pub cattle: u32,
    pub food: u32,
    pub begging: u32,
}

impl Farm {
    pub fn new() -> Self {
        Farm {
            house: House::Wood,
            rooms: 2,
            family: 2,
            fields: Vec::new(),
            pastures: Vec::new(),
            loose_stables: 0,
            wood: 0, clay: 0, reed: 0, stone: 0, grain: 0, veg: 0,
            sheep: 0, boar: 0, cattle: 0, food: 0, begging: 0,
        }
    }

    /// 已使用的農場格數（房 + 田 + 牧場格 + 無圈畜舍）。
    pub fn used_tiles(&self) -> u8 {
        let pasture_tiles: u8 = self.pastures.iter().map(|p| p.tiles).sum();
        self.rooms + self.fields.len() as u8 + pasture_tiles + self.loose_stables
    }

    pub fn free_tiles(&self) -> u8 {
        FARM_TILES.saturating_sub(self.used_tiles())
    }
}

impl Default for Farm {
    fn default() -> Self {
        Self::new()
    }
}

// ===== 行動格 =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // 累積格（每輪未取則堆積）
    Forest,    // 木
    ClayPit,   // 土
    Reed,      // 蘆葦
    Quarry,    // 石
    River,     // 糧（釣魚）
    SheepPen,  // 羊
    BoarPen,   // 豬
    CattlePen, // 牛
    // 固定格
    GrainSeeds, // +1 穀
    VegSeeds,   // +1 菜
    Plow,       // +1 田
    Sow,        // 播種
    Fences,     // 圍牧場
    BuildRooms, // 蓋房（+畜舍）
    Renovate,   // 翻修房屋材質
    FamilyGrowth, // +1 家庭成員
    DayLabor,   // +2 糧
    StartPlayer, // 取先手 + 1 糧
}

impl Action {
    pub fn as_str(self) -> &'static str {
        match self {
            Action::Forest => "forest",
            Action::ClayPit => "clay_pit",
            Action::Reed => "reed",
            Action::Quarry => "quarry",
            Action::River => "river",
            Action::SheepPen => "sheep_pen",
            Action::BoarPen => "boar_pen",
            Action::CattlePen => "cattle_pen",
            Action::GrainSeeds => "grain_seeds",
            Action::VegSeeds => "veg_seeds",
            Action::Plow => "plow",
            Action::Sow => "sow",
            Action::Fences => "fences",
            Action::BuildRooms => "build_rooms",
            Action::Renovate => "renovate",
            Action::FamilyGrowth => "family_growth",
            Action::DayLabor => "day_labor",
            Action::StartPlayer => "start_player",
        }
    }

    pub fn from_str(s: &str) -> Option<Action> {
        Some(match s {
            "forest" => Action::Forest,
            "clay_pit" => Action::ClayPit,
            "reed" => Action::Reed,
            "quarry" => Action::Quarry,
            "river" => Action::River,
            "sheep_pen" => Action::SheepPen,
            "boar_pen" => Action::BoarPen,
            "cattle_pen" => Action::CattlePen,
            "grain_seeds" => Action::GrainSeeds,
            "veg_seeds" => Action::VegSeeds,
            "plow" => Action::Plow,
            "sow" => Action::Sow,
            "fences" => Action::Fences,
            "build_rooms" => Action::BuildRooms,
            "renovate" => Action::Renovate,
            "family_growth" => Action::FamilyGrowth,
            "day_labor" => Action::DayLabor,
            "start_player" => Action::StartPlayer,
            _ => return None,
        })
    }
}

impl Crop {
    pub fn as_str(self) -> &'static str {
        match self {
            Crop::Grain => "grain",
            Crop::Vegetable => "vegetable",
        }
    }
}

impl Animal {
    pub fn as_str(self) -> &'static str {
        match self {
            Animal::Sheep => "sheep",
            Animal::Boar => "boar",
            Animal::Cattle => "cattle",
        }
    }
}

impl House {
    pub fn as_str(self) -> &'static str {
        match self {
            House::Wood => "wood",
            House::Clay => "clay",
            House::Stone => "stone",
        }
    }
}

/// 永遠可用的基礎行動格。
pub const BASE_ACTIONS: [Action; 8] = [
    Action::Forest,
    Action::ClayPit,
    Action::River,
    Action::GrainSeeds,
    Action::Plow,
    Action::DayLabor,
    Action::StartPlayer,
    Action::BuildRooms,
];

/// 逐輪揭示的行動格（round R 揭示 index R-1，超出則該輪不新增）。
pub const ROUND_ACTIONS: [Action; 11] = [
    Action::Reed,
    Action::SheepPen,
    Action::Sow,
    Action::Fences,
    Action::FamilyGrowth,
    Action::BoarPen,
    Action::Renovate,
    Action::VegSeeds,
    Action::Quarry,
    Action::CattlePen,
    Action::FamilyGrowth, // 末段重複一次成長機會（不同格視為佔位，phase-1 簡化）
];

fn is_accumulation(a: Action) -> bool {
    matches!(
        a,
        Action::Forest | Action::ClayPit | Action::Reed | Action::Quarry
            | Action::River | Action::SheepPen | Action::BoarPen | Action::CattlePen
    )
}

/// 累積格每輪的進帳量。
fn accum_increment(a: Action) -> u32 {
    match a {
        Action::Forest => 3,
        Action::ClayPit => 1,
        Action::Reed => 1,
        Action::Quarry => 1,
        Action::River => 1,
        Action::SheepPen => 1,
        Action::BoarPen => 1,
        Action::CattlePen => 1,
        _ => 0,
    }
}

/// 行動參數（依 action 取用對應欄位）。
#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    pub grain_fields: u8,    // Sow：以穀播種的田數
    pub veg_fields: u8,      // Sow：以菜播種的田數
    pub rooms: u8,           // BuildRooms：蓋房數
    pub stables: u8,         // BuildRooms：同時蓋畜舍數
    pub pasture_tiles: u8,   // Fences：牧場格數
    pub pasture_stable: bool, // Fences：是否含畜舍
}

// ===== 對局狀態 =====

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Placing,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub players: Vec<Farm>,
    pub n: usize,
    pub round: u8,
    pub phase: Phase,
    pub starting_player: usize,
    order: Vec<usize>,           // 本輪放置順序（先手起）
    cursor: usize,               // order 索引
    workers_left: Vec<u8>,       // 本輪各家剩餘工人
    occupied: HashSet<Action>,   // 本輪已佔格
    unlocked: HashSet<Action>,   // 已揭示格
    accum: HashMap<Action, u32>, // 累積格存量
    next_start: Option<usize>,   // 本輪有人搶先手
}

pub fn initial_state(n: usize) -> GameState {
    let players = (0..n).map(|_| Farm::new()).collect();
    let mut gs = GameState {
        players,
        n,
        round: 0,
        phase: Phase::Placing,
        starting_player: 0,
        order: Vec::new(),
        cursor: 0,
        workers_left: vec![0; n],
        occupied: HashSet::new(),
        unlocked: BASE_ACTIONS.iter().copied().collect(),
        accum: HashMap::new(),
        next_start: None,
    };
    begin_round(&mut gs);
    gs
}

fn begin_round(gs: &mut GameState) {
    gs.round += 1;
    // 揭示本輪行動格
    if let Some(&a) = ROUND_ACTIONS.get((gs.round - 1) as usize) {
        gs.unlocked.insert(a);
    }
    // 累積格進帳
    for &a in gs.unlocked.iter() {
        if is_accumulation(a) {
            *gs.accum.entry(a).or_insert(0) += accum_increment(a);
        }
    }
    gs.occupied.clear();
    gs.workers_left = gs.players.iter().map(|f| f.family).collect();
    gs.starting_player = gs.next_start.take().unwrap_or(gs.starting_player);
    gs.order = (0..gs.n).map(|i| (gs.starting_player + i) % gs.n).collect();
    gs.cursor = 0;
    advance_cursor(gs);
}

/// 將 cursor 移到下一個尚有工人的玩家。
fn advance_cursor(gs: &mut GameState) {
    let mut steps = 0;
    while steps < gs.n {
        let p = gs.order[gs.cursor % gs.n];
        if gs.workers_left[p] > 0 {
            gs.cursor %= gs.n;
            return;
        }
        gs.cursor += 1;
        steps += 1;
    }
}

/// 當前該下工人的玩家（None = 本輪結束/已結算）。
pub fn current_player(gs: &GameState) -> Option<usize> {
    if gs.phase != Phase::Placing {
        return None;
    }
    if gs.workers_left.iter().all(|&w| w == 0) {
        return None;
    }
    Some(gs.order[gs.cursor % gs.n])
}

/// 累積格目前存量（供前端顯示）。
pub fn accumulation(gs: &GameState) -> Vec<(Action, u32)> {
    let mut v: Vec<(Action, u32)> = gs
        .accum
        .iter()
        .filter(|(_, &amt)| amt > 0)
        .map(|(&a, &amt)| (a, amt))
        .collect();
    v.sort_by_key(|(a, _)| format!("{a:?}"));
    v
}

/// 本輪仍可選的行動格（已揭示且未被佔）。Phase-2 hub 會用。
pub fn available_actions(gs: &GameState) -> Vec<Action> {
    let mut v: Vec<Action> = gs
        .unlocked
        .iter()
        .copied()
        .filter(|a| !gs.occupied.contains(a))
        .collect();
    v.sort_by_key(|a| format!("{a:?}"));
    v
}

// ===== 動作執行 =====

/// 玩家在 `action` 放工人。驗證 → 套用 → 推進輪次（必要時收穫/進下一輪）。
pub fn take_action(gs: &mut GameState, player: usize, action: Action, input: Input) -> Result<(), &'static str> {
    if gs.phase != Phase::Placing {
        return Err("game_over");
    }
    if current_player(gs) != Some(player) {
        return Err("not_your_turn");
    }
    if !gs.unlocked.contains(&action) {
        return Err("locked");
    }
    if gs.occupied.contains(&action) {
        return Err("occupied");
    }

    // 在 farm 副本上套用結構性效果，成功才寫回（保持失敗無副作用）
    let mut farm = gs.players[player].clone();
    let accum_amt = gs.accum.get(&action).copied().unwrap_or(0);
    apply_action(&mut farm, action, accum_amt, input)?;
    gs.players[player] = farm;

    // 副作用：累積格清零、先手、佔格
    if is_accumulation(action) {
        gs.accum.insert(action, 0);
    }
    if action == Action::StartPlayer {
        gs.next_start = Some(player);
    }
    gs.occupied.insert(action);

    // 推進
    gs.workers_left[player] -= 1;
    gs.cursor += 1;
    advance_cursor(gs);
    if gs.workers_left.iter().all(|&w| w == 0) {
        end_round(gs);
    }
    Ok(())
}

fn animal_capacity(f: &Farm) -> u32 {
    let pasture_cap: u32 = f
        .pastures
        .iter()
        .map(|p| if p.stable { p.tiles as u32 * 4 } else { p.tiles as u32 * 2 })
        .sum();
    pasture_cap + f.loose_stables as u32 + 1 // 自宅可養 1
}

/// 取得動物入欄；超出容量的部分就地「煮食」換糧。
fn gain_animals(f: &mut Farm, animal: Animal, amt: u32) {
    let cap = animal_capacity(f);
    let slot = match animal {
        Animal::Sheep => &mut f.sheep,
        Animal::Boar => &mut f.boar,
        Animal::Cattle => &mut f.cattle,
    };
    *slot += amt;
    let total = f.sheep + f.boar + f.cattle;
    if total > cap {
        let excess = total - cap;
        let food_val = match animal {
            Animal::Sheep | Animal::Boar => 2,
            Animal::Cattle => 3,
        };
        let slot = match animal {
            Animal::Sheep => &mut f.sheep,
            Animal::Boar => &mut f.boar,
            Animal::Cattle => &mut f.cattle,
        };
        let cooked = excess.min(*slot);
        *slot -= cooked;
        f.food += cooked * food_val;
    }
}

fn apply_action(f: &mut Farm, action: Action, accum: u32, input: Input) -> Result<(), &'static str> {
    match action {
        Action::Forest => f.wood += accum,
        Action::ClayPit => f.clay += accum,
        Action::Reed => f.reed += accum,
        Action::Quarry => f.stone += accum,
        Action::River => f.food += accum,
        Action::SheepPen => gain_animals(f, Animal::Sheep, accum),
        Action::BoarPen => gain_animals(f, Animal::Boar, accum),
        Action::CattlePen => gain_animals(f, Animal::Cattle, accum),
        Action::GrainSeeds => f.grain += 1,
        Action::VegSeeds => f.veg += 1,
        Action::DayLabor => f.food += 2,
        Action::StartPlayer => f.food += 1,
        Action::Plow => {
            if f.free_tiles() < 1 {
                return Err("no_space");
            }
            f.fields.push(Field::default());
        }
        Action::Sow => {
            let empties = f.fields.iter().filter(|x| x.crop.is_none()).count() as u8;
            if input.grain_fields + input.veg_fields > empties {
                return Err("not_enough_fields");
            }
            if f.grain < input.grain_fields as u32 || f.veg < input.veg_fields as u32 {
                return Err("no_seeds");
            }
            let (mut g, mut v) = (input.grain_fields, input.veg_fields);
            for field in f.fields.iter_mut() {
                if field.crop.is_some() {
                    continue;
                }
                if g > 0 {
                    field.crop = Some((Crop::Grain, 3));
                    g -= 1;
                } else if v > 0 {
                    field.crop = Some((Crop::Vegetable, 2));
                    v -= 1;
                }
            }
            f.grain -= input.grain_fields as u32;
            f.veg -= input.veg_fields as u32;
        }
        Action::Fences => {
            let t = input.pasture_tiles;
            if t < 1 {
                return Err("bad_pasture");
            }
            if f.free_tiles() < t {
                return Err("no_space");
            }
            let cost = t as u32 + 1 + if input.pasture_stable { 2 } else { 0 };
            if f.wood < cost {
                return Err("no_wood");
            }
            f.wood -= cost;
            f.pastures.push(Pasture { tiles: t, stable: input.pasture_stable, animal: None });
        }
        Action::BuildRooms => {
            let rc = input.rooms;
            let sc = input.stables;
            if f.free_tiles() < rc + sc {
                return Err("no_space");
            }
            let (mat_cost, reed_cost) = (5 * rc as u32, 2 * rc as u32);
            let ok_mat = match f.house {
                House::Wood => f.wood >= mat_cost,
                House::Clay => f.clay >= mat_cost,
                House::Stone => f.stone >= mat_cost,
            };
            if !ok_mat || f.reed < reed_cost || f.wood < 2 * sc as u32 {
                return Err("no_materials");
            }
            match f.house {
                House::Wood => f.wood -= mat_cost,
                House::Clay => f.clay -= mat_cost,
                House::Stone => f.stone -= mat_cost,
            }
            f.reed -= reed_cost;
            f.wood -= 2 * sc as u32;
            f.rooms += rc;
            f.loose_stables += sc;
        }
        Action::Renovate => match f.house {
            House::Wood => {
                if f.clay < f.rooms as u32 || f.reed < 1 {
                    return Err("no_materials");
                }
                f.clay -= f.rooms as u32;
                f.reed -= 1;
                f.house = House::Clay;
            }
            House::Clay => {
                if f.stone < f.rooms as u32 || f.reed < 1 {
                    return Err("no_materials");
                }
                f.stone -= f.rooms as u32;
                f.reed -= 1;
                f.house = House::Stone;
            }
            House::Stone => return Err("max_house"),
        },
        Action::FamilyGrowth => {
            if f.family >= f.rooms {
                return Err("no_room");
            }
            f.family += 1;
        }
    }
    Ok(())
}

// ===== 輪末 / 收穫 =====

fn end_round(gs: &mut GameState) {
    if HARVEST_ROUNDS.contains(&gs.round) {
        for f in gs.players.iter_mut() {
            harvest(f);
        }
    }
    if gs.round >= TOTAL_ROUNDS {
        gs.phase = Phase::GameOver;
    } else {
        begin_round(gs);
    }
}

/// 收穫：① 田收成 ② 餵食（不足以資源/牲畜煮食補，再不足乞討）③ 牲畜繁殖。
pub fn harvest(f: &mut Farm) {
    // ① 田收成：每塊有作物的田產 1 單位入庫
    for field in f.fields.iter_mut() {
        if let Some((crop, count)) = field.crop {
            match crop {
                Crop::Grain => f.grain += 1,
                Crop::Vegetable => f.veg += 1,
            }
            let left = count - 1;
            field.crop = if left == 0 { None } else { Some((crop, left)) };
        }
    }
    // ② 餵食
    let mut need = f.family as u32 * 2;
    let pay = need.min(f.food);
    f.food -= pay;
    need -= pay;
    while need > 0 && f.grain > 0 {
        f.grain -= 1;
        need -= 1;
    }
    while need > 0 && f.veg > 0 {
        f.veg -= 1;
        need -= 1;
    }
    for (count, val) in [(&mut f.sheep, 2u32), (&mut f.boar, 2), (&mut f.cattle, 3)] {
        while need > 0 && *count > 0 {
            *count -= 1;
            need = need.saturating_sub(val);
        }
    }
    if need > 0 {
        f.begging += need; // 每缺 1 糧 = 1 乞討（計分 -3）
    }
    // ③ 繁殖：每種 ≥2 且還有容量 → +1
    let cap = animal_capacity(f);
    for animal in [Animal::Sheep, Animal::Boar, Animal::Cattle] {
        let total = f.sheep + f.boar + f.cattle;
        if total >= cap {
            break;
        }
        let slot = match animal {
            Animal::Sheep => &mut f.sheep,
            Animal::Boar => &mut f.boar,
            Animal::Cattle => &mut f.cattle,
        };
        if *slot >= 2 {
            *slot += 1;
        }
    }
}

// ===== 計分 =====

fn bucket(n: u32, t1: u32, t2: u32, t3: u32, t4: u32) -> i32 {
    if n == 0 {
        -1
    } else if n < t1 {
        1
    } else if n < t2 {
        2
    } else if n < t3 {
        3
    } else if n >= t4 {
        4
    } else {
        3
    }
}

/// 單一農場最終分數。
pub fn score(f: &Farm) -> i32 {
    let grain_total = f.grain
        + f.fields.iter().filter_map(|x| match x.crop {
            Some((Crop::Grain, c)) => Some(c as u32),
            _ => None,
        }).sum::<u32>();
    let veg_total = f.veg
        + f.fields.iter().filter_map(|x| match x.crop {
            Some((Crop::Vegetable, c)) => Some(c as u32),
            _ => None,
        }).sum::<u32>();

    let mut s = 0i32;
    // 田數：0-1→-1,2→1,3→2,4→3,5+→4
    s += match f.fields.len() {
        0 | 1 => -1,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 4,
    };
    // 牧場數：0→-1,1→1,2→2,3→3,4+→4
    s += match f.pastures.len() {
        0 => -1,
        1 => 1,
        2 => 2,
        3 => 3,
        _ => 4,
    };
    // 穀：0→-1,1-3→1,4-5→2,6-7→3,8+→4
    s += bucket(grain_total, 4, 6, 8, 8);
    // 菜：0→-1,1→1,2→2,3→3,4+→4
    s += match veg_total {
        0 => -1,
        1 => 1,
        2 => 2,
        3 => 3,
        _ => 4,
    };
    // 羊：1-3→1,4-5→2,6-7→3,8+→4
    s += bucket(f.sheep, 4, 6, 8, 8);
    // 豬：1-2→1,3-4→2,5-6→3,7+→4
    s += bucket(f.boar, 3, 5, 7, 7);
    // 牛：1→1,2-3→2,4-5→3,6+→4
    s += bucket(f.cattle, 2, 4, 6, 6);
    // 有圈畜舍：每個 +1
    s += f.pastures.iter().filter(|p| p.stable).count() as i32;
    // 房間材質：土 +1、石 +2（木 0）
    s += match f.house {
        House::Wood => 0,
        House::Clay => f.rooms as i32,
        House::Stone => f.rooms as i32 * 2,
    };
    // 家庭成員 +3
    s += f.family as i32 * 3;
    // 乞討 -3
    s -= f.begging as i32 * 3;
    // 空格 -1
    s -= f.free_tiles() as i32;
    s
}

#[allow(dead_code)]
pub fn final_scores(gs: &GameState) -> Vec<i32> {
    gs.players.iter().map(score).collect()
}

#[cfg(test)]
mod tests;

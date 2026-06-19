//! 阿瓦隆角色、陣營、人數配置表、私有可見資訊。純資料/純函式。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Merlin,       // 好：看得見壞人（除莫德雷）
    Percival,     // 好：看得見梅林與莫甘娜（無法分辨）
    LoyalServant, // 好：無資訊
    Assassin,     // 壞：結局猜梅林
    Morgana,      // 壞：對派西維爾顯示為梅林
    Mordred,      // 壞：梅林看不見他（開關）
    Oberon,       // 壞：與其他壞人互不知（開關）
    Minion,       // 壞：無特殊
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Good,
    Evil,
}

impl Role {
    pub fn alignment(self) -> Alignment {
        match self {
            Role::Merlin | Role::Percival | Role::LoyalServant => Alignment::Good,
            _ => Alignment::Evil,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Role::Merlin => "merlin",
            Role::Percival => "percival",
            Role::LoyalServant => "loyal_servant",
            Role::Assassin => "assassin",
            Role::Morgana => "morgana",
            Role::Mordred => "mordred",
            Role::Oberon => "oberon",
            Role::Minion => "minion",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub mordred: bool,
    pub oberon: bool,
}

/// 各人數的 (好人, 壞人) 數。
pub fn good_evil(n: usize) -> Option<(usize, usize)> {
    Some(match n {
        5 => (3, 2),
        6 => (4, 2),
        7 => (4, 3),
        8 => (5, 3),
        9 => (6, 3),
        10 => (6, 4),
        _ => return None,
    })
}

/// 各人數五輪任務的上場人數。
pub fn quest_sizes(n: usize) -> Option<[usize; 5]> {
    Some(match n {
        5 => [2, 3, 2, 3, 3],
        6 => [2, 3, 4, 3, 4],
        7 => [2, 3, 3, 4, 4],
        8 | 9 | 10 => [3, 4, 4, 5, 5],
        _ => return None,
    })
}

/// 第 4 輪（round index 3）且 7 人以上 → 需 2 張失敗票才算任務失敗。
pub fn fails_required(n: usize, round: usize) -> usize {
    if n >= 7 && round == 3 {
        2
    } else {
        1
    }
}

/// 依人數 + 開關組出角色清單（未洗牌）。壞人特殊角色超過壞人數 → Err。
pub fn build_roles(n: usize, opt: Options) -> Result<Vec<Role>, &'static str> {
    let (good, evil) = good_evil(n).ok_or("bad_player_count")?;

    let mut evils = vec![Role::Assassin, Role::Morgana];
    if opt.mordred {
        evils.push(Role::Mordred);
    }
    if opt.oberon {
        evils.push(Role::Oberon);
    }
    if evils.len() > evil {
        return Err("too_many_special_evil");
    }
    while evils.len() < evil {
        evils.push(Role::Minion);
    }

    let mut goods = vec![Role::Merlin, Role::Percival];
    while goods.len() < good {
        goods.push(Role::LoyalServant);
    }

    let mut all = goods;
    all.extend(evils);
    Ok(all)
}

/// `seat` 看得見的座位（意義依其角色）：
/// - 梅林 → 所有壞人座位（除莫德雷）
/// - 派西維爾 → 梅林與莫甘娜的座位
/// - 壞人（非奧伯倫）→ 其他壞人座位（除奧伯倫，不含自己）
/// - 其餘 → 空
pub fn known_seats(roles: &[Role], seat: usize) -> Vec<usize> {
    let me = roles[seat];
    let mut out = Vec::new();
    for (i, &r) in roles.iter().enumerate() {
        if i == seat {
            continue;
        }
        let visible = match me {
            Role::Merlin => r.alignment() == Alignment::Evil && r != Role::Mordred,
            Role::Percival => r == Role::Merlin || r == Role::Morgana,
            Role::Assassin | Role::Morgana | Role::Mordred | Role::Minion => {
                r.alignment() == Alignment::Evil && r != Role::Oberon
            }
            _ => false, // LoyalServant / Oberon
        };
        if visible {
            out.push(i);
        }
    }
    out
}

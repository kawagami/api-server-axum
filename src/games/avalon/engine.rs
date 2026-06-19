//! 阿瓦隆狀態機 — 純函式，零 WS 依賴。組隊 → 投票 → 任務 → （刺客）→ 結算。

use super::roles::{self, Alignment, Options, Role};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    TeamBuilding, // 隊長提名
    TeamVote,     // 全員公開投票
    Quest,        // 上場者暗投成功/失敗
    Assassinate,  // 好人完成 3 任務 → 刺客猜梅林
    GameOver,
}

#[derive(Debug, Clone)]
pub struct AvalonState {
    pub roles: Vec<Role>,
    pub n: usize,
    pub sizes: [usize; 5],
    pub leader: usize,
    pub round: usize,        // 當前任務序（0..5）
    pub results: Vec<bool>,  // 已完成任務結果（true=成功）
    pub rejects: usize,      // 連續否決次數
    pub phase: Phase,
    pub team: Vec<usize>,    // 提名/上場座位
    pub votes: Vec<Option<bool>>, // 組隊投票（每座位）
    pub cards: Vec<Option<bool>>, // 任務牌（僅上場者）
    pub winner: Option<Alignment>,
    pub reason: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VoteTally {
    pub votes: Vec<(usize, bool)>, // 公開：誰贊成/否決
    pub approved: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QuestTally {
    pub round: usize,
    pub fails: usize, // 失敗票數（匿名，只回計數）
    pub success: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssassinResult {
    pub target: usize,
    pub correct: bool,
}

/// 以指定角色排列建局（測試用，決定論）。
pub fn setup_with_roles(roles: Vec<Role>, leader: usize) -> AvalonState {
    let n = roles.len();
    let sizes = roles::quest_sizes(n).expect("valid n");
    AvalonState {
        roles,
        n,
        sizes,
        leader,
        round: 0,
        results: Vec::new(),
        rejects: 0,
        phase: Phase::TeamBuilding,
        team: Vec::new(),
        votes: vec![None; n],
        cards: vec![None; n],
        winner: None,
        reason: "",
    }
}

/// 正式建局：組角色、洗牌、隨機隊長。
pub fn setup(n: usize, opt: Options) -> Result<AvalonState, &'static str> {
    use rand::seq::SliceRandom;
    let mut roles = roles::build_roles(n, opt)?;
    let mut rng = rand::thread_rng();
    roles.shuffle(&mut rng);
    let leader = (rand::random::<u64>() % n as u64) as usize;
    Ok(setup_with_roles(roles, leader))
}

pub fn current_quest_size(s: &AvalonState) -> usize {
    s.sizes[s.round]
}

fn in_range(s: &AvalonState, seat: usize) -> bool {
    seat < s.n
}

/// 隊長提名上場隊伍。
pub fn propose_team(s: &mut AvalonState, leader: usize, team: &[usize]) -> Result<(), &'static str> {
    if s.phase != Phase::TeamBuilding {
        return Err("wrong_phase");
    }
    if leader != s.leader {
        return Err("not_leader");
    }
    if team.len() != current_quest_size(s) {
        return Err("bad_team_size");
    }
    let mut seen = std::collections::HashSet::new();
    for &t in team {
        if !in_range(s, t) || !seen.insert(t) {
            return Err("bad_team");
        }
    }
    s.team = team.to_vec();
    s.votes = vec![None; s.n];
    s.phase = Phase::TeamVote;
    Ok(())
}

/// 組隊投票。回傳 None 表尚未投完；Some 表已結算。
pub fn team_vote(s: &mut AvalonState, seat: usize, approve: bool) -> Result<Option<VoteTally>, &'static str> {
    if s.phase != Phase::TeamVote {
        return Err("wrong_phase");
    }
    if !in_range(s, seat) {
        return Err("bad_seat");
    }
    s.votes[seat] = Some(approve);
    if s.votes.iter().any(|v| v.is_none()) {
        return Ok(None);
    }
    let votes: Vec<(usize, bool)> = s.votes.iter().map(|v| v.unwrap()).enumerate().collect();
    let approvals = votes.iter().filter(|(_, a)| *a).count();
    let approved = approvals * 2 > s.n; // 嚴格多數（平手＝否決）

    if approved {
        s.rejects = 0;
        s.cards = vec![None; s.n];
        s.phase = Phase::Quest;
    } else {
        s.rejects += 1;
        if s.rejects >= 5 {
            s.winner = Some(Alignment::Evil);
            s.reason = "evil_five_rejects";
            s.phase = Phase::GameOver;
        } else {
            s.leader = (s.leader + 1) % s.n;
            s.phase = Phase::TeamBuilding;
        }
    }
    Ok(Some(VoteTally { votes, approved }))
}

/// 任務暗投。好人不得投失敗。回傳 None 表尚未投完；Some 表任務結算。
pub fn quest_card(s: &mut AvalonState, seat: usize, success: bool) -> Result<Option<QuestTally>, &'static str> {
    if s.phase != Phase::Quest {
        return Err("wrong_phase");
    }
    if !s.team.contains(&seat) {
        return Err("not_on_team");
    }
    if s.roles[seat].alignment() == Alignment::Good && !success {
        return Err("good_must_succeed");
    }
    s.cards[seat] = Some(success);
    if s.team.iter().any(|&t| s.cards[t].is_none()) {
        return Ok(None);
    }

    let fails = s.team.iter().filter(|&&t| s.cards[t] == Some(false)).count();
    let req = roles::fails_required(s.n, s.round);
    let success_quest = fails < req;
    s.results.push(success_quest);

    let tally = QuestTally { round: s.round, fails, success: success_quest };

    let successes = s.results.iter().filter(|&&r| r).count();
    let failures = s.results.len() - successes;
    if successes >= 3 {
        s.phase = Phase::Assassinate; // 設定中恆有刺客
    } else if failures >= 3 {
        s.winner = Some(Alignment::Evil);
        s.reason = "evil_three_fails";
        s.phase = Phase::GameOver;
    } else {
        s.leader = (s.leader + 1) % s.n;
        s.round += 1;
        s.team.clear();
        s.phase = Phase::TeamBuilding;
    }
    Ok(Some(tally))
}

/// 刺客猜梅林。
pub fn assassinate(s: &mut AvalonState, seat: usize, target: usize) -> Result<AssassinResult, &'static str> {
    if s.phase != Phase::Assassinate {
        return Err("wrong_phase");
    }
    if s.roles[seat] != Role::Assassin {
        return Err("not_assassin");
    }
    if !in_range(s, target) {
        return Err("bad_target");
    }
    let correct = s.roles[target] == Role::Merlin;
    s.winner = Some(if correct { Alignment::Evil } else { Alignment::Good });
    s.reason = if correct { "evil_assassinate" } else { "good_assassin_miss" };
    s.phase = Phase::GameOver;
    Ok(AssassinResult { target, correct })
}

#[cfg(test)]
mod tests;

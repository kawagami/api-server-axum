//! 阿瓦隆 WS 串接：私有角色推送 / 階段機 / 投票 / 任務 / 刺客 / chat / 斷線。
//!
//! 信封 `{ game:"avalon", type, data }`。角色資訊**逐座位私有推送**（各人看到不同 `known`）。
//! 大廳 / 房間共通指令（join_lobby / list_rooms / create_room / join_room / leave_room）
//! 走 `common::room` 泛型框架。

use std::net::SocketAddr;

use serde_json::{json, Value};

use super::engine::{self, AvalonState, Phase};
use super::hub::{AvalonHub, AvalonHubInner, AvalonRoom, Room, RoomState};
use super::roles::{self, Alignment};
use crate::games::common::room;
use crate::state::AppState;

pub async fn handle(hub: &AvalonHub, state: &AppState, who: SocketAddr, value: &Value) -> bool {
    let data = value.get("data");
    let Some(typ) = value.get("type").and_then(|v| v.as_str()) else { return false };
    if room::handle_common(hub, state, who, typ, data).await {
        return true;
    }
    match typ {
        "start_game" => start_game(hub, state, who).await,
        "chat" => chat(hub, state, who, data).await,
        "propose_team" => propose_team(hub, state, who, data).await,
        "team_vote" => team_vote(hub, state, who, data).await,
        "quest_card" => quest_card(hub, state, who, data).await,
        "assassinate" => assassinate(hub, state, who, data).await,
        _ => return false,
    }
    true
}

pub async fn handle_disconnect(hub: &AvalonHub, state: &AppState, who: SocketAddr) {
    room::handle_disconnect(hub, state, who).await;
}

fn msg(typ: &str, data: Value) -> String {
    room::msg::<AvalonRoom>(typ, data)
}

fn err1(state: &AppState, who: SocketAddr, reason: &str) {
    room::err1::<AvalonRoom>(state, who, reason);
}

// ---- 開局 ----

async fn start_game(hub: &AvalonHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let room_id = match room::start_check(&h, who) {
            Ok(id) => id,
            Err(e) => { err1(state, who, e); return; }
        };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let n = room.players.len();
        let st = match engine::setup(n, room.options) {
            Ok(st) => st,
            Err(e) => { err1(state, who, e); return; }
        };
        // 私有角色推送：每座位收到自己的角色 + known
        for seat in 0..n {
            let role = st.roles[seat];
            let known = roles::known_seats(&st.roles, seat);
            let players: Vec<Value> = room.names.iter().enumerate()
                .map(|(i, name)| json!({ "seat": i, "name": name })).collect();
            outbox.push((
                room.players[seat],
                msg("role_assigned", json!({
                    "your_seat": seat,
                    "your_role": role.as_str(),
                    "known": known,
                    "n": n,
                    "sizes": st.sizes,
                    "players": players,
                })),
            ));
        }
        room.state = RoomState::Playing(st);
        // 公開階段
        let room = h.rooms.get(&room_id).unwrap();
        broadcast_phase(room, &mut outbox);
        room::push_lobby_update(&h, &mut outbox);
    }
    room::flush(state, outbox);
}

async fn chat(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let text = data.and_then(|d| d.get("text")).and_then(|v| v.as_str()).unwrap_or("").trim();
    if text.is_empty() { return; }
    let text: String = text.chars().take(500).collect();
    let mut outbox = Vec::new();
    {
        let h = hub.lock().await;
        let Some(&room_id) = h.conn_room.get(&who) else { return; };
        let room = h.rooms.get(&room_id).unwrap();
        let Some(seat) = room.seat_of(who) else { return; };
        let m = msg("chat", json!({ "seat": seat, "name": room.names[seat], "text": text }));
        for &p in &room.players { outbox.push((p, m.clone())); }
    }
    room::flush(state, outbox);
}

// ---- 對局動作 ----

async fn propose_team(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let team: Vec<usize> = match data.and_then(|d| d.get("team")).and_then(|v| v.as_array()) {
        Some(arr) => arr.iter().filter_map(|x| x.as_u64().map(|u| u as usize)).collect(),
        None => { err1(state, who, "bad_team"); return; }
    };
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some((room_id, seat)) = room::playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let RoomState::Playing(st) = &mut room.state else { return; };
        if let Err(e) = engine::propose_team(st, seat, &team) { err1(state, who, e); return; }
        let m = msg("team_proposed", json!({ "team": team, "leader": seat }));
        for &p in &room.players { outbox.push((p, m.clone())); }
        broadcast_phase(room, &mut outbox);
    }
    room::flush(state, outbox);
}

async fn team_vote(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let Some(approve) = data.and_then(|d| d.get("approve")).and_then(|v| v.as_bool()) else {
        err1(state, who, "bad_vote"); return;
    };
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some((room_id, seat)) = room::playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let RoomState::Playing(st) = &mut room.state else { return; };
        match engine::team_vote(st, seat, approve) {
            Err(e) => { err1(state, who, e); return; }
            Ok(None) => {} // 尚未投完，靜默
            Ok(Some(tally)) => {
                let votes: Vec<Value> = tally.votes.iter().map(|(s, a)| json!({ "seat": s, "approve": a })).collect();
                let m = msg("vote_result", json!({ "votes": votes, "approved": tally.approved }));
                for &p in &room.players { outbox.push((p, m.clone())); }
                push_transition(&mut h, room_id, &mut outbox);
            }
        }
    }
    room::flush(state, outbox);
}

async fn quest_card(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let Some(success) = data.and_then(|d| d.get("success")).and_then(|v| v.as_bool()) else {
        err1(state, who, "bad_card"); return;
    };
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some((room_id, seat)) = room::playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let RoomState::Playing(st) = &mut room.state else { return; };
        match engine::quest_card(st, seat, success) {
            Err(e) => { err1(state, who, e); return; }
            Ok(None) => {}
            Ok(Some(tally)) => {
                let m = msg("quest_result", json!({
                    "round": tally.round, "fails": tally.fails, "success": tally.success,
                }));
                for &p in &room.players { outbox.push((p, m.clone())); }
                push_transition(&mut h, room_id, &mut outbox);
            }
        }
    }
    room::flush(state, outbox);
}

async fn assassinate(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let Some(target) = data.and_then(|d| d.get("target")).and_then(|v| v.as_u64()).map(|u| u as usize) else {
        err1(state, who, "bad_target"); return;
    };
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some((room_id, seat)) = room::playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let RoomState::Playing(st) = &mut room.state else { return; };
        if let Err(e) = engine::assassinate(st, seat, target) { err1(state, who, e); return; }
        push_transition(&mut h, room_id, &mut outbox); // phase 已 GameOver → 推 game_over + 解散
    }
    room::flush(state, outbox);
}

// ---- 階段推送 ----

fn phase_str(p: Phase) -> &'static str {
    match p {
        Phase::TeamBuilding => "team_building",
        Phase::TeamVote => "team_vote",
        Phase::Quest => "quest",
        Phase::Assassinate => "assassinate",
        Phase::GameOver => "game_over",
    }
}

fn phase_payload(st: &AvalonState) -> Value {
    json!({
        "phase": phase_str(st.phase),
        "leader": st.leader,
        "round": st.round,
        "quest_size": st.sizes.get(st.round).copied().unwrap_or(0),
        "results": st.results,
        "rejects": st.rejects,
        "team": st.team,
    })
}

fn broadcast_phase(room: &Room, outbox: &mut Vec<(SocketAddr, String)>) {
    if let RoomState::Playing(st) = &room.state {
        let m = msg("phase_changed", phase_payload(st));
        for &p in &room.players {
            outbox.push((p, m.clone()));
        }
    }
}

/// 動作後依 phase 推 phase_changed，或 GameOver → 推 game_over（含全角色揭露）並解散房。
fn push_transition(h: &mut AvalonHubInner, room_id: u64, outbox: &mut Vec<(SocketAddr, String)>) {
    let room = h.rooms.get(&room_id).unwrap();
    let RoomState::Playing(st) = &room.state else { return; };
    if st.phase == Phase::GameOver {
        let roles_reveal: Vec<Value> = st.roles.iter().enumerate()
            .map(|(i, r)| json!({ "seat": i, "role": r.as_str() })).collect();
        let winner = match st.winner {
            Some(Alignment::Good) => json!("good"),
            Some(Alignment::Evil) => json!("evil"),
            None => Value::Null,
        };
        let m = msg("game_over", json!({ "winner": winner, "reason": st.reason, "roles": roles_reveal }));
        for &p in &room.players { outbox.push((p, m.clone())); }
        room::dissolve_room(h, room_id, outbox);
    } else {
        broadcast_phase(room, outbox);
    }
}

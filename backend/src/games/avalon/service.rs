//! 阿瓦隆 WS 串接：大廳 / N 人房 / 私有角色推送 / 階段機 / 投票 / 任務 / 刺客 / chat / 斷線。
//!
//! 信封 `{ game:"avalon", type, data }`。角色資訊**逐座位私有推送**（各人看到不同 `known`）。

use std::net::SocketAddr;

use serde_json::{json, Value};

use super::engine::{self, AvalonState, Phase};
use super::hub::{AvalonHub, AvalonHubInner, Room, RoomState, MAX_PLAYERS};
use super::roles::{self, Alignment, Options};
use crate::state::AppState;

pub async fn handle(hub: &AvalonHub, state: &AppState, who: SocketAddr, value: &Value) -> bool {
    let data = value.get("data");
    match value.get("type").and_then(|v| v.as_str()) {
        Some("join_lobby") => join_lobby(hub, state, who).await,
        Some("list_rooms") => list_rooms(hub, state, who).await,
        Some("create_room") => create_room(hub, state, who, data).await,
        Some("join_room") => join_room(hub, state, who, data).await,
        Some("leave_room") => leave_room(hub, state, who).await,
        Some("start_game") => start_game(hub, state, who).await,
        Some("chat") => chat(hub, state, who, data).await,
        Some("propose_team") => propose_team(hub, state, who, data).await,
        Some("team_vote") => team_vote(hub, state, who, data).await,
        Some("quest_card") => quest_card(hub, state, who, data).await,
        Some("assassinate") => assassinate(hub, state, who, data).await,
        _ => return false,
    }
    true
}

fn msg(typ: &str, data: Value) -> String {
    crate::structs::ws::game_envelope(super::NAME, typ, data)
}

fn flush(state: &AppState, outbox: Vec<(SocketAddr, String)>) {
    for (addr, m) in outbox {
        state.send_to(addr, m);
    }
}

fn err1(state: &AppState, who: SocketAddr, reason: &str) {
    state.send_to(who, msg("error", json!({ "reason": reason })));
}

// ---- 大廳 ----

fn lobby_snapshot(hub: &AvalonHubInner) -> Value {
    let mut rooms: Vec<&Room> = hub.rooms.values().collect();
    rooms.sort_by_key(|r| r.id);
    let list: Vec<Value> = rooms
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "players": r.players.len(),
                "max": MAX_PLAYERS,
                "status": if matches!(r.state, RoomState::Waiting) { "waiting" } else { "playing" },
            })
        })
        .collect();
    json!({ "rooms": list })
}

fn push_lobby_update(hub: &AvalonHubInner, outbox: &mut Vec<(SocketAddr, String)>) {
    let m = msg("lobby_update", lobby_snapshot(hub));
    for &addr in &hub.lobby {
        outbox.push((addr, m.clone()));
    }
}

fn room_snapshot(room: &Room) -> Value {
    let players: Vec<Value> = room
        .players
        .iter()
        .enumerate()
        .map(|(seat, _)| json!({ "seat": seat, "name": room.names[seat] }))
        .collect();
    json!({
        "room_id": room.id,
        "name": room.name,
        "host_seat": room.seat_of(room.host),
        "players": players,
        "options": { "mordred": room.options.mordred, "oberon": room.options.oberon },
        "can_start": room.can_start(),
    })
}

fn push_room_update(room: &Room, outbox: &mut Vec<(SocketAddr, String)>) {
    let m = msg("room_update", room_snapshot(room));
    for &p in &room.players {
        outbox.push((p, m.clone()));
    }
}

async fn join_lobby(hub: &AvalonHub, state: &AppState, who: SocketAddr) {
    let mut h = hub.lock().await;
    h.lobby.insert(who);
    let snap = msg("room_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snap);
}

async fn list_rooms(hub: &AvalonHub, state: &AppState, who: SocketAddr) {
    let h = hub.lock().await;
    let snap = msg("room_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snap);
}

// ---- 房間 ----

async fn create_room(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if h.is_committed(who) {
            err1(state, who, "already_committed");
            return;
        }
        let id = h.next_id;
        h.next_id += 1;
        let name = data
            .and_then(|d| d.get("room_name"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.chars().take(40).collect::<String>())
            .unwrap_or_else(|| format!("房 #{id}"));
        let options = Options {
            mordred: data.and_then(|d| d.pointer("/options/mordred")).and_then(|v| v.as_bool()).unwrap_or(false),
            oberon: data.and_then(|d| d.pointer("/options/oberon")).and_then(|v| v.as_bool()).unwrap_or(false),
        };
        let pname = nickname(data).unwrap_or_else(|| "玩家1".to_string()); // host = seat 0 → 顯示玩家1
        h.rooms.insert(
            id,
            Room { id, name, players: vec![who], names: vec![pname], host: who, options, state: RoomState::Waiting },
        );
        h.conn_room.insert(who, id);
        outbox.push((who, msg("room_created", json!({ "room_id": id }))));
        push_room_update(h.rooms.get(&id).unwrap(), &mut outbox);
        push_lobby_update(&h, &mut outbox);
    }
    flush(state, outbox);
}

async fn join_room(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if h.is_committed(who) {
            err1(state, who, "already_committed");
            return;
        }
        let Some(room_id) = data.and_then(|d| d.get("room_id")).and_then(|v| v.as_u64()) else {
            err1(state, who, "bad_room_id");
            return;
        };
        let nick = nickname(data);
        match h.rooms.get_mut(&room_id) {
            None => { err1(state, who, "room_not_found"); return; }
            Some(room) => {
                if !matches!(room.state, RoomState::Waiting) { err1(state, who, "already_started"); return; }
                if room.is_full() { err1(state, who, "room_full"); return; }
                let seat = room.players.len();
                room.players.push(who);
                room.names.push(nick.unwrap_or_else(|| format!("玩家{}", seat + 1)));
            }
        }
        h.conn_room.insert(who, room_id);
        push_room_update(h.rooms.get(&room_id).unwrap(), &mut outbox);
        push_lobby_update(&h, &mut outbox);
    }
    flush(state, outbox);
}

async fn leave_room(hub: &AvalonHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        remove_from_room(&mut h, who, "host_left", &mut outbox);
    }
    flush(state, outbox);
}

async fn start_game(hub: &AvalonHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some(&room_id) = h.conn_room.get(&who) else { err1(state, who, "not_in_room"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        if room.host != who { err1(state, who, "not_host"); return; }
        if !room.can_start() { err1(state, who, "cannot_start"); return; }
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
        push_lobby_update(&h, &mut outbox);
    }
    flush(state, outbox);
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
    flush(state, outbox);
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
        let Some((room_id, seat)) = playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let RoomState::Playing(st) = &mut room.state else { return; };
        if let Err(e) = engine::propose_team(st, seat, &team) { err1(state, who, e); return; }
        let m = msg("team_proposed", json!({ "team": team, "leader": seat }));
        for &p in &room.players { outbox.push((p, m.clone())); }
        broadcast_phase(room, &mut outbox);
    }
    flush(state, outbox);
}

async fn team_vote(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let Some(approve) = data.and_then(|d| d.get("approve")).and_then(|v| v.as_bool()) else {
        err1(state, who, "bad_vote"); return;
    };
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some((room_id, seat)) = playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
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
    flush(state, outbox);
}

async fn quest_card(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let Some(success) = data.and_then(|d| d.get("success")).and_then(|v| v.as_bool()) else {
        err1(state, who, "bad_card"); return;
    };
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some((room_id, seat)) = playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
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
    flush(state, outbox);
}

async fn assassinate(hub: &AvalonHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let Some(target) = data.and_then(|d| d.get("target")).and_then(|v| v.as_u64()).map(|u| u as usize) else {
        err1(state, who, "bad_target"); return;
    };
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some((room_id, seat)) = playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let RoomState::Playing(st) = &mut room.state else { return; };
        if let Err(e) = engine::assassinate(st, seat, target) { err1(state, who, e); return; }
        push_transition(&mut h, room_id, &mut outbox); // phase 已 GameOver → 推 game_over + 解散
    }
    flush(state, outbox);
}

// ---- 斷線 ----

pub async fn handle_disconnect(hub: &AvalonHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        h.lobby.remove(&who);
        remove_from_room(&mut h, who, "player_left", &mut outbox);
    }
    flush(state, outbox);
}

// ---- 共用 ----

/// 玩家暱稱（`nickname` 欄），trim、≤20 字；空則回 None 由呼叫端帶預設。
fn nickname(data: Option<&Value>) -> Option<String> {
    data.and_then(|d| d.get("nickname"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(20).collect())
}

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
        // 解散房
        let players = room.players.clone();
        h.rooms.remove(&room_id);
        for p in players { h.conn_room.remove(&p); }
        push_lobby_update(h, outbox);
    } else {
        broadcast_phase(room, outbox);
    }
}

/// 取得 (room_id, seat)，限對局中的房。
fn playing_seat(h: &AvalonHubInner, who: SocketAddr) -> Option<(u64, usize)> {
    let &room_id = h.conn_room.get(&who)?;
    let room = h.rooms.get(&room_id)?;
    if !matches!(room.state, RoomState::Playing(_)) {
        return None;
    }
    let seat = room.seat_of(who)?;
    Some((room_id, seat))
}

/// 離開房：等待中移除玩家（host 離開＝解散）；對局中＝中止整局並解散。
fn remove_from_room(h: &mut AvalonHubInner, who: SocketAddr, _why: &str, outbox: &mut Vec<(SocketAddr, String)>) {
    let Some(&room_id) = h.conn_room.get(&who) else { return; };
    let Some(room) = h.rooms.get(&room_id) else { return; };
    let playing = matches!(room.state, RoomState::Playing(_));
    let is_host = room.host == who;

    if playing || is_host {
        // 中止整局 / host 解散 → 通知其餘並解散
        let reason = if playing { "aborted" } else { "host_left" };
        let players = room.players.clone();
        let m = msg("room_closed", json!({ "reason": reason }));
        for &p in &players {
            if p != who {
                outbox.push((p, m.clone()));
            }
        }
        h.rooms.remove(&room_id);
        for p in players {
            h.conn_room.remove(&p);
        }
        push_lobby_update(h, outbox);
    } else {
        // 等待中的非 host 玩家離開：移除座位
        let room = h.rooms.get_mut(&room_id).unwrap();
        if let Some(idx) = room.seat_of(who) {
            room.players.remove(idx);
            room.names.remove(idx);
        }
        h.conn_room.remove(&who);
        let room = h.rooms.get(&room_id).unwrap();
        push_room_update(room, outbox);
        push_lobby_update(h, outbox);
    }
}

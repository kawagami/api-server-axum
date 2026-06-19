//! 農場經營 WS 串接：大廳 / N 人房 / 動作 / 完整狀態廣播 / 斷線。
//!
//! 信封 `{ game:"farm", type, data }`。**完全資訊**：每次動作後廣播全盤狀態給房內所有人（無私有推送）。

use std::net::SocketAddr;

use serde_json::{json, Value};

use super::engine::{self, Action, Farm, GameState, Input, Phase};
use super::hub::{FarmHub, FarmHubInner, Room, RoomState, MAX_PLAYERS};
use crate::state::AppState;

pub async fn handle(hub: &FarmHub, state: &AppState, who: SocketAddr, value: &Value) -> bool {
    let data = value.get("data");
    match value.get("type").and_then(|v| v.as_str()) {
        Some("join_lobby") => join_lobby(hub, state, who).await,
        Some("list_rooms") => list_rooms(hub, state, who).await,
        Some("create_room") => create_room(hub, state, who, data).await,
        Some("join_room") => join_room(hub, state, who, data).await,
        Some("leave_room") => leave_room(hub, state, who).await,
        Some("start_game") => start_game(hub, state, who).await,
        Some("action") => action(hub, state, who, data).await,
        _ => return false,
    }
    true
}

fn msg(typ: &str, data: Value) -> String {
    crate::structs::ws::game_envelope("farm", typ, data)
}

fn flush(state: &AppState, outbox: Vec<(SocketAddr, String)>) {
    for (addr, m) in outbox {
        state.send_to(addr, m);
    }
}

fn err1(state: &AppState, who: SocketAddr, reason: &str) {
    state.send_to(who, msg("error", json!({ "reason": reason })));
}

fn nickname(data: Option<&Value>) -> Option<String> {
    data.and_then(|d| d.get("nickname"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(20).collect())
}

// ---- 大廳 ----

fn lobby_snapshot(hub: &FarmHubInner) -> Value {
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

fn push_lobby_update(hub: &FarmHubInner, outbox: &mut Vec<(SocketAddr, String)>) {
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
        "can_start": room.can_start(),
    })
}

fn push_room_update(room: &Room, outbox: &mut Vec<(SocketAddr, String)>) {
    let base = room_snapshot(room);
    // 逐人注入 your_seat（等待中有人離開會重編號，故每則都帶當前 seat）
    for (seat, &p) in room.players.iter().enumerate() {
        let mut v = base.clone();
        v["your_seat"] = json!(seat);
        outbox.push((p, msg("room_update", v)));
    }
}

async fn join_lobby(hub: &FarmHub, state: &AppState, who: SocketAddr) {
    let mut h = hub.lock().await;
    h.lobby.insert(who);
    let snap = msg("room_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snap);
}

async fn list_rooms(hub: &FarmHub, state: &AppState, who: SocketAddr) {
    let h = hub.lock().await;
    let snap = msg("room_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snap);
}

// ---- 房間 ----

async fn create_room(hub: &FarmHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
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
            .unwrap_or_else(|| format!("農場 #{id}"));
        let pname = nickname(data).unwrap_or_else(|| "玩家1".to_string());
        h.rooms.insert(
            id,
            Room { id, name, players: vec![who], names: vec![pname], host: who, state: RoomState::Waiting },
        );
        h.conn_room.insert(who, id);
        outbox.push((who, msg("room_created", json!({ "room_id": id }))));
        push_room_update(h.rooms.get(&id).unwrap(), &mut outbox);
        push_lobby_update(&h, &mut outbox);
    }
    flush(state, outbox);
}

async fn join_room(hub: &FarmHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
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

async fn leave_room(hub: &FarmHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        remove_from_room(&mut h, who, &mut outbox);
    }
    flush(state, outbox);
}

async fn start_game(hub: &FarmHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some(&room_id) = h.conn_room.get(&who) else { err1(state, who, "not_in_room"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        if room.host != who { err1(state, who, "not_host"); return; }
        if !room.can_start() { err1(state, who, "cannot_start"); return; }
        let gs = engine::initial_state(room.players.len());
        room.state = RoomState::Playing(gs);
        let room = h.rooms.get(&room_id).unwrap();
        broadcast_state(room, &mut outbox);
        push_lobby_update(&h, &mut outbox);
    }
    flush(state, outbox);
}

// ---- 對局動作 ----

async fn action(hub: &FarmHub, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let Some(action) = data
        .and_then(|d| d.get("action"))
        .and_then(|v| v.as_str())
        .and_then(Action::from_str)
    else {
        err1(state, who, "bad_action");
        return;
    };
    let input = parse_input(data.and_then(|d| d.get("input")));

    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some(&room_id) = h.conn_room.get(&who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let Some(seat) = room.seat_of(who) else { err1(state, who, "not_in_game"); return; };
        let RoomState::Playing(gs) = &mut room.state else { err1(state, who, "not_in_game"); return; };

        if let Err(e) = engine::take_action(gs, seat, action, input) {
            err1(state, who, e);
            return;
        }

        let over = gs.phase == Phase::GameOver;
        let room = h.rooms.get(&room_id).unwrap();
        broadcast_state(room, &mut outbox);
        if over {
            // game_over 帶最終分數，並解散房
            if let RoomState::Playing(gs) = &room.state {
                let scores = engine::final_scores(gs);
                let m = msg("game_over", json!({ "scores": scores }));
                for &p in &room.players { outbox.push((p, m.clone())); }
            }
            let players = room.players.clone();
            h.rooms.remove(&room_id);
            for p in players { h.conn_room.remove(&p); }
            push_lobby_update(&h, &mut outbox);
        }
    }
    flush(state, outbox);
}

// ---- 斷線 ----

pub async fn handle_disconnect(hub: &FarmHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        h.lobby.remove(&who);
        remove_from_room(&mut h, who, &mut outbox);
    }
    flush(state, outbox);
}

// ---- 共用 ----

fn parse_input(v: Option<&Value>) -> Input {
    let g = |k: &str| v.and_then(|d| d.get(k)).and_then(|x| x.as_u64()).unwrap_or(0) as u8;
    Input {
        grain_fields: g("grain_fields"),
        veg_fields: g("veg_fields"),
        rooms: g("rooms"),
        stables: g("stables"),
        pasture_tiles: g("pasture_tiles"),
        pasture_stable: v.and_then(|d| d.get("pasture_stable")).and_then(|x| x.as_bool()).unwrap_or(false),
    }
}

fn farm_json(f: &Farm) -> Value {
    let fields: Vec<Value> = f.fields.iter().map(|fl| match fl.crop {
        Some((c, n)) => json!({ "crop": c.as_str(), "count": n }),
        None => Value::Null,
    }).collect();
    let pastures: Vec<Value> = f.pastures.iter().map(|p| {
        json!({
            "tiles": p.tiles,
            "stable": p.stable,
            "animal": p.animal.map(|(a, n)| json!({ "kind": a.as_str(), "count": n })),
        })
    }).collect();
    json!({
        "house": f.house.as_str(),
        "rooms": f.rooms,
        "family": f.family,
        "fields": fields,
        "pastures": pastures,
        "loose_stables": f.loose_stables,
        "free_tiles": f.free_tiles(),
        "wood": f.wood, "clay": f.clay, "reed": f.reed, "stone": f.stone,
        "grain": f.grain, "veg": f.veg,
        "sheep": f.sheep, "boar": f.boar, "cattle": f.cattle,
        "food": f.food, "begging": f.begging,
    })
}

fn state_payload(gs: &GameState) -> Value {
    let players: Vec<Value> = gs.players.iter().map(farm_json).collect();
    let actions: Vec<&str> = engine::available_actions(gs).iter().map(|a| a.as_str()).collect();
    let accum: Vec<Value> = engine::accumulation(gs).iter()
        .map(|(a, n)| json!({ "action": a.as_str(), "amount": n })).collect();
    json!({
        "round": gs.round,
        "phase": if gs.phase == Phase::GameOver { "game_over" } else { "placing" },
        "current_player": engine::current_player(gs),
        "starting_player": gs.starting_player,
        "players": players,
        "available_actions": actions,
        "accumulation": accum,
    })
}

fn broadcast_state(room: &Room, outbox: &mut Vec<(SocketAddr, String)>) {
    if let RoomState::Playing(gs) = &room.state {
        let base = state_payload(gs);
        // 逐人注入 your_seat
        for (seat, &p) in room.players.iter().enumerate() {
            let mut v = base.clone();
            v["your_seat"] = json!(seat);
            outbox.push((p, msg("state", v)));
        }
    }
}

/// 離開房：等待中移除玩家（host 離開＝解散）；對局中＝中止整局並解散。
fn remove_from_room(h: &mut FarmHubInner, who: SocketAddr, outbox: &mut Vec<(SocketAddr, String)>) {
    let Some(&room_id) = h.conn_room.get(&who) else { return; };
    let Some(room) = h.rooms.get(&room_id) else { return; };
    let playing = matches!(room.state, RoomState::Playing(_));
    let is_host = room.host == who;

    if playing || is_host {
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

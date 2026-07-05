//! 農場經營 WS 串接：動作 / 完整狀態廣播 / 斷線。
//!
//! 信封 `{ game:"farm", type, data }`。**完全資訊**：每次動作後廣播全盤狀態給房內所有人（無私有推送）。
//! 大廳 / 房間共通指令（join_lobby / list_rooms / create_room / join_room / leave_room）
//! 走 `common::room` 泛型框架。

use std::net::SocketAddr;

use serde_json::{json, Value};

use super::engine::{self, Action, Farm, GameState, Input, Phase};
use super::hub::{FarmHub, FarmRoom, Room, RoomState};
use crate::games::common::room;
use crate::state::AppState;

pub async fn handle(hub: &FarmHub, state: &AppState, who: SocketAddr, value: &Value) -> bool {
    let data = value.get("data");
    let Some(typ) = value.get("type").and_then(|v| v.as_str()) else { return false };
    if room::handle_common(hub, state, who, typ, data).await {
        return true;
    }
    match typ {
        "start_game" => start_game(hub, state, who).await,
        "action" => action(hub, state, who, data).await,
        _ => return false,
    }
    true
}

pub async fn handle_disconnect(hub: &FarmHub, state: &AppState, who: SocketAddr) {
    room::handle_disconnect(hub, state, who).await;
}

fn msg(typ: &str, data: Value) -> String {
    room::msg::<FarmRoom>(typ, data)
}

fn err1(state: &AppState, who: SocketAddr, reason: &str) {
    room::err1::<FarmRoom>(state, who, reason);
}

// ---- 開局 ----

async fn start_game(hub: &FarmHub, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let room_id = match room::start_check(&h, who) {
            Ok(id) => id,
            Err(e) => { err1(state, who, e); return; }
        };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let gs = engine::initial_state(room.players.len());
        room.state = RoomState::Playing(gs);
        let room = h.rooms.get(&room_id).unwrap();
        broadcast_state(room, &mut outbox);
        room::push_lobby_update(&h, &mut outbox);
    }
    room::flush(state, outbox);
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
        let Some((room_id, seat)) = room::playing_seat(&h, who) else { err1(state, who, "not_in_game"); return; };
        let room = h.rooms.get_mut(&room_id).unwrap();
        let RoomState::Playing(gs) = &mut room.state else { return; };

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
            room::dissolve_room(&mut h, room_id, &mut outbox);
        }
    }
    room::flush(state, outbox);
}

// ---- 狀態廣播 ----

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

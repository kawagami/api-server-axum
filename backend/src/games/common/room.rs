//! N 人房共用框架：大廳 / 房間 CRUD / 座位 / 斷線，泛型於 `RoomKind`。
//!
//! 2 人對戰走 `common::{hub,service}`（桌位 + 配對 + 計時）；N 人子系統（avalon / farm）
//! 走這裡：各遊戲以 marker type impl `RoomKind` 提供靜態參數，遊戲專屬的開局 / 對局
//! 動作 / 廣播留在各自 `service.rs`，共通指令由 `handle_common` 統一分派。

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;

use serde_json::{json, Map, Value};
use tokio::sync::Mutex;

use crate::state::AppState;

/// N 人房遊戲的靜態參數。各遊戲在 `hub.rs` 以 marker type 實作。
pub trait RoomKind: Send + Sized + 'static {
    /// 對局中的完整狀態（engine state）。
    type Playing: Send;
    /// 建房選項；無選項用 `()`。
    type Options: Default + Send;

    /// 遊戲代號（registry key / WS 信封 `game` 欄）。
    const NAME: &'static str;
    const MIN_PLAYERS: usize;
    const MAX_PLAYERS: usize;
    /// `room_update` 是否逐人注入 `your_seat`（等待中有人離開會重編號）。
    const SEAT_IN_ROOM_UPDATE: bool = false;

    /// 未命名房的預設房名。
    fn default_room_name(id: u64) -> String;

    /// 從 `create_room` 的 data 解析選項。
    fn parse_options(_data: Option<&Value>) -> Self::Options {
        Default::default()
    }

    /// `room_snapshot` 的附加欄位（如阿瓦隆的 `options`）。
    fn extend_room_snapshot(_options: &Self::Options, _obj: &mut Map<String, Value>) {}
}

pub type RoomHub<K> = Arc<Mutex<RoomHubInner<K>>>;

pub struct RoomHubInner<K: RoomKind> {
    pub rooms: HashMap<u64, Room<K>>,
    pub conn_room: HashMap<SocketAddr, u64>,
    pub lobby: HashSet<SocketAddr>,
    pub next_id: u64,
}

// 手寫 Default：derive 會多要求 K: Default。
impl<K: RoomKind> Default for RoomHubInner<K> {
    fn default() -> Self {
        Self { rooms: HashMap::new(), conn_room: HashMap::new(), lobby: HashSet::new(), next_id: 0 }
    }
}

impl<K: RoomKind> RoomHubInner<K> {
    pub fn is_committed(&self, who: SocketAddr) -> bool {
        self.conn_room.contains_key(&who)
    }
}

pub struct Room<K: RoomKind> {
    pub id: u64,
    pub name: String,
    /// 座位順序＝加入順序；對局開始後固定。index = seat。
    pub players: Vec<SocketAddr>,
    pub names: Vec<String>,
    pub host: SocketAddr,
    pub options: K::Options,
    pub state: RoomState<K::Playing>,
}

// 房間數少、Waiting 短命，Playing 大 payload 不 Box 也無記憶體壓力
#[allow(clippy::large_enum_variant)]
pub enum RoomState<S> {
    Waiting,
    Playing(S),
}

impl<K: RoomKind> Room<K> {
    pub fn seat_of(&self, who: SocketAddr) -> Option<usize> {
        self.players.iter().position(|&p| p == who)
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= K::MAX_PLAYERS
    }

    pub fn can_start(&self) -> bool {
        matches!(self.state, RoomState::Waiting)
            && (K::MIN_PLAYERS..=K::MAX_PLAYERS).contains(&self.players.len())
    }
}

// ---- 訊息 ----

pub fn msg<K: RoomKind>(typ: &str, data: Value) -> String {
    crate::structs::ws::game_envelope(K::NAME, typ, data)
}

pub fn flush(state: &AppState, outbox: Vec<(SocketAddr, String)>) {
    for (addr, m) in outbox {
        state.send_to(addr, m);
    }
}

pub fn err1<K: RoomKind>(state: &AppState, who: SocketAddr, reason: &str) {
    state.send_to(who, msg::<K>("error", json!({ "reason": reason })));
}

/// 玩家暱稱（`nickname` 欄），trim、≤20 字；空則回 None 由呼叫端帶預設。
pub fn nickname(data: Option<&Value>) -> Option<String> {
    data.and_then(|d| d.get("nickname"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(20).collect())
}

// ---- 大廳 ----

pub fn lobby_snapshot<K: RoomKind>(hub: &RoomHubInner<K>) -> Value {
    let mut rooms: Vec<&Room<K>> = hub.rooms.values().collect();
    rooms.sort_by_key(|r| r.id);
    let list: Vec<Value> = rooms
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "players": r.players.len(),
                "max": K::MAX_PLAYERS,
                "status": if matches!(r.state, RoomState::Waiting) { "waiting" } else { "playing" },
            })
        })
        .collect();
    json!({ "rooms": list })
}

pub fn push_lobby_update<K: RoomKind>(hub: &RoomHubInner<K>, outbox: &mut Vec<(SocketAddr, String)>) {
    let m = msg::<K>("lobby_update", lobby_snapshot(hub));
    for &addr in &hub.lobby {
        outbox.push((addr, m.clone()));
    }
}

pub fn room_snapshot<K: RoomKind>(room: &Room<K>) -> Value {
    let players: Vec<Value> = room
        .players
        .iter()
        .enumerate()
        .map(|(seat, _)| json!({ "seat": seat, "name": room.names[seat] }))
        .collect();
    let mut v = json!({
        "room_id": room.id,
        "name": room.name,
        "host_seat": room.seat_of(room.host),
        "players": players,
        "can_start": room.can_start(),
    });
    if let Value::Object(obj) = &mut v {
        K::extend_room_snapshot(&room.options, obj);
    }
    v
}

pub fn push_room_update<K: RoomKind>(room: &Room<K>, outbox: &mut Vec<(SocketAddr, String)>) {
    let base = room_snapshot(room);
    if K::SEAT_IN_ROOM_UPDATE {
        // 逐人注入 your_seat（等待中有人離開會重編號，故每則都帶當前 seat）
        for (seat, &p) in room.players.iter().enumerate() {
            let mut v = base.clone();
            v["your_seat"] = json!(seat);
            outbox.push((p, msg::<K>("room_update", v)));
        }
    } else {
        let m = msg::<K>("room_update", base);
        for &p in &room.players {
            outbox.push((p, m.clone()));
        }
    }
}

// ---- 共通指令 ----

/// 分派大廳 / 房間共通指令；非共通指令回 `false` 由各遊戲 service 接手。
pub async fn handle_common<K: RoomKind>(
    hub: &RoomHub<K>,
    state: &AppState,
    who: SocketAddr,
    typ: &str,
    data: Option<&Value>,
) -> bool {
    match typ {
        "join_lobby" => join_lobby(hub, state, who).await,
        "list_rooms" => list_rooms(hub, state, who).await,
        "create_room" => create_room(hub, state, who, data).await,
        "join_room" => join_room(hub, state, who, data).await,
        "leave_room" => leave_room(hub, state, who).await,
        _ => return false,
    }
    true
}

pub async fn join_lobby<K: RoomKind>(hub: &RoomHub<K>, state: &AppState, who: SocketAddr) {
    let mut h = hub.lock().await;
    h.lobby.insert(who);
    let snap = msg::<K>("room_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snap);
}

pub async fn list_rooms<K: RoomKind>(hub: &RoomHub<K>, state: &AppState, who: SocketAddr) {
    let h = hub.lock().await;
    let snap = msg::<K>("room_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snap);
}

pub async fn create_room<K: RoomKind>(hub: &RoomHub<K>, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if h.is_committed(who) {
            err1::<K>(state, who, "already_committed");
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
            .unwrap_or_else(|| K::default_room_name(id));
        let options = K::parse_options(data);
        let pname = nickname(data).unwrap_or_else(|| "玩家1".to_string()); // host = seat 0 → 顯示玩家1
        h.rooms.insert(
            id,
            Room { id, name, players: vec![who], names: vec![pname], host: who, options, state: RoomState::Waiting },
        );
        h.conn_room.insert(who, id);
        outbox.push((who, msg::<K>("room_created", json!({ "room_id": id }))));
        push_room_update(h.rooms.get(&id).unwrap(), &mut outbox);
        push_lobby_update(&h, &mut outbox);
    }
    flush(state, outbox);
}

pub async fn join_room<K: RoomKind>(hub: &RoomHub<K>, state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if h.is_committed(who) {
            err1::<K>(state, who, "already_committed");
            return;
        }
        let Some(room_id) = data.and_then(|d| d.get("room_id")).and_then(|v| v.as_u64()) else {
            err1::<K>(state, who, "bad_room_id");
            return;
        };
        let nick = nickname(data);
        match h.rooms.get_mut(&room_id) {
            None => { err1::<K>(state, who, "room_not_found"); return; }
            Some(room) => {
                if !matches!(room.state, RoomState::Waiting) { err1::<K>(state, who, "already_started"); return; }
                if room.is_full() { err1::<K>(state, who, "room_full"); return; }
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

pub async fn leave_room<K: RoomKind>(hub: &RoomHub<K>, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        remove_from_room(&mut h, who, &mut outbox);
    }
    flush(state, outbox);
}

pub async fn handle_disconnect<K: RoomKind>(hub: &RoomHub<K>, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        h.lobby.remove(&who);
        remove_from_room(&mut h, who, &mut outbox);
    }
    flush(state, outbox);
}

// ---- 房內狀態工具 ----

/// 取得 (room_id, seat)，限對局中的房。
pub fn playing_seat<K: RoomKind>(h: &RoomHubInner<K>, who: SocketAddr) -> Option<(u64, usize)> {
    let &room_id = h.conn_room.get(&who)?;
    let room = h.rooms.get(&room_id)?;
    if !matches!(room.state, RoomState::Playing(_)) {
        return None;
    }
    let seat = room.seat_of(who)?;
    Some((room_id, seat))
}

/// host 開局前置檢查，通過回 room_id。
pub fn start_check<K: RoomKind>(h: &RoomHubInner<K>, who: SocketAddr) -> Result<u64, &'static str> {
    let Some(&room_id) = h.conn_room.get(&who) else { return Err("not_in_room") };
    let room = h.rooms.get(&room_id).unwrap();
    if room.host != who {
        return Err("not_host");
    }
    if !room.can_start() {
        return Err("cannot_start");
    }
    Ok(room_id)
}

/// 對局結束解散房：移除房與所有連線對應，推大廳更新。
pub fn dissolve_room<K: RoomKind>(h: &mut RoomHubInner<K>, room_id: u64, outbox: &mut Vec<(SocketAddr, String)>) {
    if let Some(room) = h.rooms.remove(&room_id) {
        for p in room.players {
            h.conn_room.remove(&p);
        }
        push_lobby_update(h, outbox);
    }
}

/// 離開房：等待中移除玩家（host 離開＝解散）；對局中＝中止整局並解散。
pub fn remove_from_room<K: RoomKind>(h: &mut RoomHubInner<K>, who: SocketAddr, outbox: &mut Vec<(SocketAddr, String)>) {
    let Some(&room_id) = h.conn_room.get(&who) else { return; };
    let Some(room) = h.rooms.get(&room_id) else { return; };
    let playing = matches!(room.state, RoomState::Playing(_));
    let is_host = room.host == who;

    if playing || is_host {
        // 中止整局 / host 解散 → 通知其餘並解散
        let reason = if playing { "aborted" } else { "host_left" };
        let players = room.players.clone();
        let m = msg::<K>("room_closed", json!({ "reason": reason }));
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

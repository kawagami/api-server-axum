//! 泛型 WS 串接：大廳 / 桌位 / 配對 / 行棋 / 計時 / 斷線。任何 `E: GameEngine` 共用。
//!
//! 信封 `{ game, type, data }`；`game` = `E::NAME`。事件點對點送相關連線，
//! 大廳更新只送 `lobby` 訂閱集。

use std::net::SocketAddr;

use serde_json::{json, Map, Value};
use tokio::time::{Duration, Instant};

use super::engine::{GameEngine, GameStatus, Side};
use super::hub::{Game, GameHub, HubInner, Table, TableState};
use crate::state::AppState;

/// 收到的 WS 文字訊息分派。回傳 true 表示已處理（呼叫端不再 echo）。
pub async fn handle<E: GameEngine>(
    hub: &GameHub<E>,
    state: &AppState,
    who: SocketAddr,
    value: &Value,
) -> bool {
    let data = value.get("data");
    match value.get("type").and_then(|v| v.as_str()) {
        Some("join_lobby") => join_lobby(hub, state, who).await,
        Some("list_tables") => list_tables(hub, state, who).await,
        Some("create_table") => create_table(hub, state, who, data).await,
        Some("join_table") => join_table(hub, state, who, data).await,
        Some("leave_table") => leave_table(hub, state, who).await,
        Some("join_queue") => join_queue(hub, state, who).await,
        Some("leave_queue") => leave_queue(hub, who).await,
        Some("resign") => resign(hub, state, who).await,
        Some("move") => handle_move(hub, state, who, data).await,
        _ => return false,
    }
    true
}

fn msg<E: GameEngine>(typ: &str, data: Value) -> String {
    json!({ "game": E::NAME, "type": typ, "data": data }).to_string()
}

fn clock_json<E: GameEngine>(game: &Game<E>) -> Value {
    let mut m = Map::new();
    m.insert(E::side_label(Side::First).to_string(), json!(game.clock_of(Side::First)));
    m.insert(E::side_label(Side::Second).to_string(), json!(game.clock_of(Side::Second)));
    Value::Object(m)
}

// ---- 大廳 ----

fn lobby_snapshot<E: GameEngine>(hub: &HubInner<E>) -> Value {
    let mut tables: Vec<&Table<E>> = hub.tables.values().collect();
    tables.sort_by_key(|t| t.id);
    let list: Vec<Value> = tables
        .iter()
        .map(|t| {
            let status = match &t.state {
                TableState::Waiting { .. } => "waiting",
                TableState::Playing(_) => "playing",
            };
            json!({ "id": t.id, "name": t.name, "status": status })
        })
        .collect();
    json!({ "tables": list })
}

fn push_lobby_update<E: GameEngine>(hub: &HubInner<E>, outbox: &mut Vec<(SocketAddr, String)>) {
    let m = msg::<E>("lobby_update", lobby_snapshot(hub));
    for &addr in &hub.lobby {
        outbox.push((addr, m.clone()));
    }
}

async fn join_lobby<E: GameEngine>(hub: &GameHub<E>, state: &AppState, who: SocketAddr) {
    let mut h = hub.lock().await;
    h.lobby.insert(who);
    let snapshot = msg::<E>("table_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snapshot);
}

async fn list_tables<E: GameEngine>(hub: &GameHub<E>, state: &AppState, who: SocketAddr) {
    let h = hub.lock().await;
    let snapshot = msg::<E>("table_list", lobby_snapshot(&h));
    drop(h);
    state.send_to(who, snapshot);
}

// ---- 桌位 ----

async fn create_table<E: GameEngine>(
    hub: &GameHub<E>,
    state: &AppState,
    who: SocketAddr,
    data: Option<&Value>,
) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if h.is_committed(who) {
            flush(state, vec![(who, msg::<E>("error", json!({ "reason": "already_committed" })))]);
            return;
        }
        let id = h.next_id;
        h.next_id += 1;
        let name = data
            .and_then(|d| d.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.chars().take(40).collect::<String>())
            .unwrap_or_else(|| format!("桌 #{id}"));

        h.tables.insert(
            id,
            Table {
                id,
                name,
                state: TableState::Waiting { host: who },
            },
        );
        h.conn_table.insert(who, id);
        outbox.push((who, msg::<E>("table_created", json!({ "table_id": id }))));
        push_lobby_update(&h, &mut outbox);
    }
    flush(state, outbox);
}

async fn join_table<E: GameEngine>(
    hub: &GameHub<E>,
    state: &AppState,
    who: SocketAddr,
    data: Option<&Value>,
) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if h.is_committed(who) {
            flush(state, vec![(who, msg::<E>("error", json!({ "reason": "already_committed" })))]);
            return;
        }
        let Some(table_id) = data.and_then(|d| d.get("table_id")).and_then(|v| v.as_u64()) else {
            flush(state, vec![(who, msg::<E>("error", json!({ "reason": "bad_table_id" })))]);
            return;
        };
        let host = match h.tables.get(&table_id) {
            Some(t) => match &t.state {
                TableState::Waiting { host } => *host,
                TableState::Playing(_) => {
                    flush(state, vec![(who, msg::<E>("error", json!({ "reason": "table_full" })))]);
                    return;
                }
            },
            None => {
                flush(state, vec![(who, msg::<E>("error", json!({ "reason": "table_not_found" })))]);
                return;
            }
        };
        if host == who {
            flush(state, vec![(who, msg::<E>("error", json!({ "reason": "cannot_join_self" })))]);
            return;
        }
        open_game(&mut h, table_id, host, who, &mut outbox);
    }
    flush(state, outbox);
}

/// host 在等待中離開 → 銷毀桌。對戰中請用 `resign`。
async fn leave_table<E: GameEngine>(hub: &GameHub<E>, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if let Some(&table_id) = h.conn_table.get(&who) {
            let is_waiting_host = matches!(
                h.tables.get(&table_id).map(|t| &t.state),
                Some(TableState::Waiting { host }) if *host == who
            );
            if is_waiting_host {
                h.tables.remove(&table_id);
                h.conn_table.remove(&who);
                push_lobby_update(&h, &mut outbox);
            }
        }
    }
    flush(state, outbox);
}

// ---- 快速配對 ----

async fn join_queue<E: GameEngine>(hub: &GameHub<E>, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        if h.is_committed(who) {
            return;
        }
        h.queue.push_back(who);
        if h.queue.len() >= 2 {
            let a = h.queue.pop_front().unwrap();
            let b = h.queue.pop_front().unwrap();
            let id = h.next_id;
            h.next_id += 1;
            h.tables.insert(
                id,
                Table {
                    id,
                    name: format!("快速對局 #{id}"),
                    state: TableState::Waiting { host: a },
                },
            );
            open_game(&mut h, id, a, b, &mut outbox);
        } else {
            outbox.push((who, msg::<E>("queued", json!({ "position": h.queue.len() }))));
        }
    }
    flush(state, outbox);
}

async fn leave_queue<E: GameEngine>(hub: &GameHub<E>, who: SocketAddr) {
    let mut h = hub.lock().await;
    h.queue.retain(|&c| c != who);
}

/// 將指定桌轉為對戰中：隨機分座位、推 match_found、退出佇列/大廳、廣播大廳更新。
fn open_game<E: GameEngine>(
    hub: &mut HubInner<E>,
    table_id: u64,
    a: SocketAddr,
    b: SocketAddr,
    outbox: &mut Vec<(SocketAddr, String)>,
) {
    let (first, second) = if rand::random::<bool>() { (a, b) } else { (b, a) };
    let game = Game::<E>::new(first, second);

    match hub.tables.get_mut(&table_id) {
        Some(table) => table.state = TableState::Playing(game),
        None => return,
    }

    hub.conn_table.insert(first, table_id);
    hub.conn_table.insert(second, table_id);
    hub.queue.retain(|&c| c != a && c != b);
    hub.lobby.remove(&a);
    hub.lobby.remove(&b);

    for side in [Side::First, Side::Second] {
        let conn = if side == Side::First { first } else { second };
        outbox.push((
            conn,
            msg::<E>(
                "match_found",
                json!({
                    "color": E::side_label(side),
                    "clock_ms": E::INITIAL_CLOCK_MS,
                    "table_id": table_id,
                }),
            ),
        ));
    }
    push_lobby_update(hub, outbox);
}

// ---- 行棋 ----

async fn handle_move<E: GameEngine>(
    hub: &GameHub<E>,
    state: &AppState,
    who: SocketAddr,
    data: Option<&Value>,
) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;

        let Some(&table_id) = h.conn_table.get(&who) else {
            flush(state, vec![(who, msg::<E>("error", json!({ "reason": "not_in_game" })))]);
            return;
        };
        let Some(game) = playing_game_mut(&mut h, table_id) else {
            flush(state, vec![(who, msg::<E>("error", json!({ "reason": "not_in_game" })))]);
            return;
        };
        if game.ended {
            flush(state, vec![(who, msg::<E>("error", json!({ "reason": "game_ended" })))]);
            return;
        }

        let side = game.side_of(who).unwrap();
        if game.engine.turn() != side {
            flush(state, vec![(who, msg::<E>("illegal_move", json!({ "reason": "NotYourTurn" })))]);
            return;
        }

        // 計時：先看行棋方是否已超時
        let now = Instant::now();
        let elapsed = now.duration_since(game.turn_started_at).as_millis() as i64;
        let remaining = game.clock_of(side) - elapsed;
        if remaining <= 0 {
            game.set_clock(side, 0);
            end_game(&mut h, table_id, Some(side.opponent()), "timeout", &mut outbox);
            flush(state, outbox);
            return;
        }

        // 合法性 + 套用
        let applied = match game.engine.try_move(side, data) {
            Err(reason) => {
                flush(state, vec![(who, msg::<E>("illegal_move", json!({ "reason": reason })))]);
                return;
            }
            Ok(a) => a,
        };

        // 落子：扣時 + Fischer 增量
        game.set_clock(side, remaining + E::INCREMENT_MS);
        game.turn_started_at = now;

        let seats = game.seats;
        let mut move_data = applied.move_data;
        if let Value::Object(map) = &mut move_data {
            map.insert("turn".into(), json!(E::side_label(game.engine.turn())));
            map.insert("clock".into(), clock_json(game));
        }
        let move_msg = msg::<E>("move_made", move_data);
        outbox.push((seats[0], move_msg.clone()));
        outbox.push((seats[1], move_msg));

        for (typ, d) in applied.extra {
            let em = msg::<E>(typ, d);
            outbox.push((seats[0], em.clone()));
            outbox.push((seats[1], em));
        }

        match game.engine.status() {
            GameStatus::Ongoing => {}
            GameStatus::Win { winner, reason } => {
                end_game(&mut h, table_id, Some(winner), reason, &mut outbox);
            }
            GameStatus::Draw { reason } => {
                end_game(&mut h, table_id, None, reason, &mut outbox);
            }
        }
    }
    flush(state, outbox);
}

async fn resign<E: GameEngine>(hub: &GameHub<E>, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        let Some(&table_id) = h.conn_table.get(&who) else {
            return;
        };
        let Some(game) = playing_game_mut(&mut h, table_id) else {
            return;
        };
        if game.ended {
            return;
        }
        let Some(side) = game.side_of(who) else {
            return;
        };
        end_game(&mut h, table_id, Some(side.opponent()), "resign", &mut outbox);
    }
    flush(state, outbox);
}

/// 連線斷開：清佇列/大廳；等待中 host 斷線銷毀桌；對戰中斷線判對手勝。
pub async fn handle_disconnect<E: GameEngine>(hub: &GameHub<E>, state: &AppState, who: SocketAddr) {
    let mut outbox = Vec::new();
    {
        let mut h = hub.lock().await;
        h.queue.retain(|&c| c != who);
        h.lobby.remove(&who);

        if let Some(&table_id) = h.conn_table.get(&who) {
            let info: Option<(bool, Option<Side>)> = h.tables.get(&table_id).map(|t| match &t.state {
                TableState::Waiting { .. } => (true, None),
                TableState::Playing(g) => {
                    let winner = if g.ended { None } else { g.side_of(who).map(|s| s.opponent()) };
                    (false, winner)
                }
            });
            match info {
                Some((true, _)) => {
                    h.tables.remove(&table_id);
                    h.conn_table.remove(&who);
                    push_lobby_update(&h, &mut outbox);
                }
                Some((false, Some(winner))) => {
                    end_game(&mut h, table_id, Some(winner), "disconnect", &mut outbox);
                }
                _ => {}
            }
        }
    }
    flush(state, outbox);
}

// ---- 共用 ----

fn playing_game_mut<E: GameEngine>(hub: &mut HubInner<E>, table_id: u64) -> Option<&mut Game<E>> {
    match hub.tables.get_mut(&table_id)?.state {
        TableState::Playing(ref mut g) => Some(g),
        TableState::Waiting { .. } => None,
    }
}

/// 推 game_over 給雙方並移除桌。`winner` 為 None 表和棋。
fn end_game<E: GameEngine>(
    hub: &mut HubInner<E>,
    table_id: u64,
    winner: Option<Side>,
    reason: &str,
    outbox: &mut Vec<(SocketAddr, String)>,
) {
    if let Some(game) = playing_game_mut(hub, table_id) {
        game.ended = true;
        let seats = game.seats;
        let winner_val = match winner {
            Some(s) => json!(E::side_label(s)),
            None => Value::Null,
        };
        let m = msg::<E>("game_over", json!({ "winner": winner_val, "reason": reason }));
        outbox.push((seats[0], m.clone()));
        outbox.push((seats[1], m));
        hub.conn_table.remove(&seats[0]);
        hub.conn_table.remove(&seats[1]);
    }
    hub.tables.remove(&table_id);
    push_lobby_update(hub, outbox);
}

fn flush(state: &AppState, outbox: Vec<(SocketAddr, String)>) {
    for (addr, m) in outbox {
        state.send_to(addr, m);
    }
}

// ---- 計時掃描 ----

/// 啟動時 spawn；每秒掃對戰中桌，當前行棋方超時即主動判負。
pub async fn timeout_watcher<E: GameEngine>(hub: GameHub<E>, state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let mut outbox = Vec::new();
        {
            let mut h = hub.lock().await;
            let now = Instant::now();
            let timed_out: Vec<(u64, Side)> = h
                .tables
                .values()
                .filter_map(|t| match &t.state {
                    TableState::Playing(g) if !g.ended => {
                        let side = g.engine.turn();
                        let elapsed = now.duration_since(g.turn_started_at).as_millis() as i64;
                        if g.clock_of(side) - elapsed <= 0 {
                            Some((t.id, side))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect();
            for (table_id, side) in timed_out {
                if let Some(game) = playing_game_mut(&mut h, table_id) {
                    game.set_clock(side, 0);
                }
                end_game(&mut h, table_id, Some(side.opponent()), "timeout", &mut outbox);
            }
        }
        flush(&state, outbox);
    }
}

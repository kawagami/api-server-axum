//! 象棋 WS 串接：大廳 / 桌位 / 配對 / 行棋 / 計時 / 斷線 — 串接 `crate::games::chess` 純引擎。
//!
//! 協定沿用 `{ type, data }`。所有訊息點對點送給相關連線（不走全域 broadcast）。
//! 大廳更新只送給訂閱 `lobby` 的連線。

use std::net::SocketAddr;

use serde_json::{json, Value};
use tokio::time::{Duration, Instant};

use crate::games::chess::engine;
use crate::games::chess::hub::{
    ChessHubInner, Game, Table, TableState, INCREMENT_MS, INITIAL_CLOCK_MS,
};
use crate::games::chess::types::{IllegalReason, Move, Side, Square, Status};
use crate::state::AppState;

/// 收到的 WS 文字訊息。回傳 true 表示已當作象棋訊息處理（呼叫端不再 echo）。
pub async fn handle(state: &AppState, who: SocketAddr, value: &Value) -> bool {
    let data = value.get("data");
    match value.get("type").and_then(|v| v.as_str()) {
        Some("join_lobby") => {
            join_lobby(state, who).await;
            true
        }
        Some("list_tables") => {
            list_tables(state, who).await;
            true
        }
        Some("create_table") => {
            create_table(state, who, data).await;
            true
        }
        Some("join_table") => {
            join_table(state, who, data).await;
            true
        }
        Some("leave_table") => {
            leave_table(state, who).await;
            true
        }
        Some("join_queue") => {
            join_queue(state, who).await;
            true
        }
        Some("leave_queue") => {
            leave_queue(state, who).await;
            true
        }
        Some("resign") => {
            resign(state, who).await;
            true
        }
        Some("move") => {
            handle_move(state, who, data).await;
            true
        }
        _ => false,
    }
}

fn msg(typ: &str, data: Value) -> String {
    json!({ "type": typ, "data": data }).to_string()
}

/// 解析 `[col, row]`，越界或非整數回 None。
fn parse_square(v: Option<&Value>) -> Option<Square> {
    let arr = v?.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    let col = arr[0].as_i64()?;
    let row = arr[1].as_i64()?;
    let sq = Square::new(col as i8, row as i8);
    if (0..=8).contains(&col) && (0..=9).contains(&row) && sq.in_bounds() {
        Some(sq)
    } else {
        None
    }
}

fn sq_json(sq: Square) -> Value {
    json!([sq.col, sq.row])
}

// ---- 大廳 ----

/// 大廳快照（依 id 排序，桌況穩定）。
fn lobby_snapshot(hub: &ChessHubInner) -> Value {
    let mut tables: Vec<&Table> = hub.tables.values().collect();
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

/// 推大廳更新給所有訂閱者。
fn push_lobby_update(hub: &ChessHubInner, outbox: &mut Vec<(SocketAddr, String)>) {
    let m = msg("lobby_update", lobby_snapshot(hub));
    for &addr in &hub.lobby {
        outbox.push((addr, m.clone()));
    }
}

async fn join_lobby(state: &AppState, who: SocketAddr) {
    let mut hub = state.chess().lock().await;
    hub.lobby.insert(who);
    let snapshot = msg("table_list", lobby_snapshot(&hub));
    drop(hub);
    state.send_to(who, snapshot);
}

async fn list_tables(state: &AppState, who: SocketAddr) {
    let hub = state.chess().lock().await;
    let snapshot = msg("table_list", lobby_snapshot(&hub));
    drop(hub);
    state.send_to(who, snapshot);
}

// ---- 桌位 ----

async fn create_table(state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        if hub.is_committed(who) {
            flush(state, vec![(who, msg("error", json!({ "reason": "already_committed" })))]);
            return;
        }
        let id = hub.next_id;
        hub.next_id += 1;
        let name = data
            .and_then(|d| d.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.chars().take(40).collect::<String>())
            .unwrap_or_else(|| format!("桌 #{id}"));

        hub.tables.insert(
            id,
            Table {
                id,
                name,
                state: TableState::Waiting { host: who },
            },
        );
        hub.conn_table.insert(who, id);

        outbox.push((who, msg("table_created", json!({ "table_id": id }))));
        push_lobby_update(&hub, &mut outbox);
    }
    flush(state, outbox);
}

async fn join_table(state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        if hub.is_committed(who) {
            flush(state, vec![(who, msg("error", json!({ "reason": "already_committed" })))]);
            return;
        }
        let Some(table_id) = data.and_then(|d| d.get("table_id")).and_then(|v| v.as_u64()) else {
            flush(state, vec![(who, msg("error", json!({ "reason": "bad_table_id" })))]);
            return;
        };
        let host = match hub.tables.get(&table_id) {
            Some(t) => match &t.state {
                TableState::Waiting { host } => *host,
                TableState::Playing(_) => {
                    flush(state, vec![(who, msg("error", json!({ "reason": "table_full" })))]);
                    return;
                }
            },
            None => {
                flush(state, vec![(who, msg("error", json!({ "reason": "table_not_found" })))]);
                return;
            }
        };
        if host == who {
            flush(state, vec![(who, msg("error", json!({ "reason": "cannot_join_self" })))]);
            return;
        }
        open_game(&mut hub, table_id, host, who, &mut outbox);
    }
    flush(state, outbox);
}

/// host 在等待中離開 → 銷毀桌。對戰中請用 `resign`。
async fn leave_table(state: &AppState, who: SocketAddr) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        if let Some(&table_id) = hub.conn_table.get(&who) {
            let is_waiting_host = matches!(
                hub.tables.get(&table_id).map(|t| &t.state),
                Some(TableState::Waiting { host }) if *host == who
            );
            if is_waiting_host {
                hub.tables.remove(&table_id);
                hub.conn_table.remove(&who);
                push_lobby_update(&hub, &mut outbox);
            }
        }
    }
    flush(state, outbox);
}

// ---- 快速配對 ----

async fn join_queue(state: &AppState, who: SocketAddr) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        if hub.is_committed(who) {
            return; // 已在桌/佇列 → 忽略
        }
        hub.queue.push_back(who);

        if hub.queue.len() >= 2 {
            let a = hub.queue.pop_front().unwrap();
            let b = hub.queue.pop_front().unwrap();
            let id = hub.next_id;
            hub.next_id += 1;
            // 先放一張臨時桌再開局（open_game 會覆寫為 Playing）
            hub.tables.insert(
                id,
                Table {
                    id,
                    name: format!("快速對局 #{id}"),
                    state: TableState::Waiting { host: a },
                },
            );
            open_game(&mut hub, id, a, b, &mut outbox);
        } else {
            outbox.push((who, msg("queued", json!({ "position": hub.queue.len() }))));
        }
    }
    flush(state, outbox);
}

async fn leave_queue(state: &AppState, who: SocketAddr) {
    let mut hub = state.chess().lock().await;
    hub.queue.retain(|&c| c != who);
}

/// 將指定桌轉為對戰中：隨機分紅黑、就座、推 match_found、退出佇列/大廳、廣播大廳更新。
fn open_game(
    hub: &mut ChessHubInner,
    table_id: u64,
    a: SocketAddr,
    b: SocketAddr,
    outbox: &mut Vec<(SocketAddr, String)>,
) {
    let (red, black) = if rand::random::<bool>() { (a, b) } else { (b, a) };
    let game = Game::new(red, black, engine::initial_state());

    if let Some(table) = hub.tables.get_mut(&table_id) {
        table.state = TableState::Playing(game);
    } else {
        return;
    }

    hub.conn_table.insert(red, table_id);
    hub.conn_table.insert(black, table_id);
    // 對戰雙方退出佇列與大廳訂閱
    hub.queue.retain(|&c| c != a && c != b);
    hub.lobby.remove(&a);
    hub.lobby.remove(&b);

    outbox.push((
        red,
        msg(
            "match_found",
            json!({ "color": "red", "clock_ms": INITIAL_CLOCK_MS, "table_id": table_id }),
        ),
    ));
    outbox.push((
        black,
        msg(
            "match_found",
            json!({ "color": "black", "clock_ms": INITIAL_CLOCK_MS, "table_id": table_id }),
        ),
    ));
    push_lobby_update(hub, outbox);
}

// ---- 行棋 ----

async fn handle_move(state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;

        let Some(&table_id) = hub.conn_table.get(&who) else {
            flush(state, vec![(who, msg("error", json!({ "reason": "not_in_game" })))]);
            return;
        };
        let Some(game) = playing_game_mut(&mut hub, table_id) else {
            flush(state, vec![(who, msg("error", json!({ "reason": "not_in_game" })))]);
            return;
        };
        if game.ended {
            flush(state, vec![(who, msg("error", json!({ "reason": "game_ended" })))]);
            return;
        }

        let side = game.side_of(who).unwrap();
        if game.state.turn != side {
            let reason = IllegalReason::NotYourTurn.code();
            flush(state, vec![(who, msg("illegal_move", json!({ "reason": reason })))]);
            return;
        }

        // 座標解析
        let (Some(from), Some(to)) = (
            parse_square(data.and_then(|d| d.get("from"))),
            parse_square(data.and_then(|d| d.get("to"))),
        ) else {
            flush(state, vec![(who, msg("illegal_move", json!({ "reason": "bad_coord" })))]);
            return;
        };

        // 計時：先看行棋方是否已超時
        let now = Instant::now();
        let elapsed = now.duration_since(game.turn_started_at).as_millis() as i64;
        let remaining = game.clock_of(side) - elapsed;
        if remaining <= 0 {
            match side {
                Side::Red => game.red_ms = 0,
                Side::Black => game.black_ms = 0,
            }
            end_game(&mut hub, table_id, Some(side.opponent()), "timeout", &mut outbox);
            flush(state, outbox);
            return;
        }

        // 合法性
        let mv = Move { from, to };
        if let Err(reason) = engine::is_legal(&game.state, mv) {
            flush(state, vec![(who, msg("illegal_move", json!({ "reason": reason.code() })))]);
            return;
        }

        // 落子：扣時 + Fischer 增量，套用走步、切換回合
        let new_clock = remaining + INCREMENT_MS;
        match side {
            Side::Red => game.red_ms = new_clock,
            Side::Black => game.black_ms = new_clock,
        }
        game.turn_started_at = now;
        engine::apply(&mut game.state, mv);

        // 先抽出所需值，結束 game 的可變借用，才能呼叫 end_game(&mut hub)
        let (red, black) = (game.red, game.black);
        let next_turn = game.state.turn;
        let (red_ms, black_ms) = (game.red_ms, game.black_ms);
        let status = engine::game_status(&game.state);
        let in_check =
            matches!(status, Status::Ongoing) && engine::is_in_check(&game.state, next_turn);

        let move_msg = msg(
            "move_made",
            json!({
                "from": sq_json(from),
                "to": sq_json(to),
                "turn": next_turn.as_str(),
                "clock": { "red": red_ms, "black": black_ms },
            }),
        );
        outbox.push((red, move_msg.clone()));
        outbox.push((black, move_msg));

        if in_check {
            let check_msg = msg("check", json!({ "side": next_turn.as_str() }));
            outbox.push((red, check_msg.clone()));
            outbox.push((black, check_msg));
        }

        match status {
            Status::Ongoing => {}
            Status::Checkmate { winner } => {
                end_game(&mut hub, table_id, Some(winner), "checkmate", &mut outbox);
            }
            Status::Stalemate { loser } => {
                end_game(&mut hub, table_id, Some(loser.opponent()), "stalemate", &mut outbox);
            }
            Status::Draw => {
                end_game(&mut hub, table_id, None, "draw_60", &mut outbox);
            }
        }
    }
    flush(state, outbox);
}

async fn resign(state: &AppState, who: SocketAddr) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        let Some(&table_id) = hub.conn_table.get(&who) else {
            return;
        };
        let Some(game) = playing_game_mut(&mut hub, table_id) else {
            return;
        };
        if game.ended {
            return;
        }
        let Some(side) = game.side_of(who) else {
            return;
        };
        end_game(&mut hub, table_id, Some(side.opponent()), "resign", &mut outbox);
    }
    flush(state, outbox);
}

/// 連線斷開：清佇列/大廳；等待中的 host 斷線銷毀桌；對戰中斷線判對手勝。
pub async fn handle_disconnect(state: &AppState, who: SocketAddr) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        hub.queue.retain(|&c| c != who);
        hub.lobby.remove(&who);

        if let Some(&table_id) = hub.conn_table.get(&who) {
            // 先判斷動作，結束對 hub.tables 的不可變借用，才能後續 mutate
            let info: Option<(bool, Option<Side>)> = hub.tables.get(&table_id).map(|t| match &t.state {
                TableState::Waiting { .. } => (true, None),
                TableState::Playing(g) => {
                    let winner = if g.ended { None } else { g.side_of(who).map(|s| s.opponent()) };
                    (false, winner)
                }
            });
            match info {
                Some((true, _)) => {
                    hub.tables.remove(&table_id);
                    hub.conn_table.remove(&who);
                    push_lobby_update(&hub, &mut outbox);
                }
                Some((false, Some(winner))) => {
                    end_game(&mut hub, table_id, Some(winner), "disconnect", &mut outbox);
                }
                _ => {}
            }
        }
    }
    flush(state, outbox);
}

// ---- 共用 ----

fn playing_game_mut(hub: &mut ChessHubInner, table_id: u64) -> Option<&mut Game> {
    match hub.tables.get_mut(&table_id)?.state {
        TableState::Playing(ref mut g) => Some(g),
        TableState::Waiting { .. } => None,
    }
}

/// 推 game_over 給雙方並移除桌。`winner` 為 None 表和棋。
fn end_game(
    hub: &mut ChessHubInner,
    table_id: u64,
    winner: Option<Side>,
    reason: &str,
    outbox: &mut Vec<(SocketAddr, String)>,
) {
    if let Some(game) = playing_game_mut(hub, table_id) {
        game.ended = true;
        let (red, black) = (game.red, game.black);
        let winner_val = match winner {
            Some(s) => json!(s.as_str()),
            None => Value::Null,
        };
        let m = msg("game_over", json!({ "winner": winner_val, "reason": reason }));
        outbox.push((red, m.clone()));
        outbox.push((black, m));
        hub.conn_table.remove(&red);
        hub.conn_table.remove(&black);
    }
    hub.tables.remove(&table_id);
    push_lobby_update(hub, outbox);
}

fn flush(state: &AppState, outbox: Vec<(SocketAddr, String)>) {
    for (addr, m) in outbox {
        state.send_to(addr, m);
    }
}

// ---- 計時掃描：偵測行棋方時鐘耗盡卻無人走步 ----

/// 啟動時 spawn；每秒掃對戰中桌，當前行棋方超時即主動判負。
pub async fn timeout_watcher(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
        {
            let mut hub = state.chess().lock().await;
            let now = Instant::now();
            let timed_out: Vec<(u64, Side)> = hub
                .tables
                .values()
                .filter_map(|t| match &t.state {
                    TableState::Playing(g) if !g.ended => {
                        let side = g.state.turn;
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
                if let Some(game) = playing_game_mut(&mut hub, table_id) {
                    match side {
                        Side::Red => game.red_ms = 0,
                        Side::Black => game.black_ms = 0,
                    }
                }
                end_game(&mut hub, table_id, Some(side.opponent()), "timeout", &mut outbox);
            }
        }
        flush(&state, outbox);
    }
}

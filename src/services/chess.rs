//! 象棋 WS 串接：配對、行棋、計時、斷線 — 串接 `crate::games::chess` 純引擎。
//!
//! 協定沿用 `{ type, data }`。所有訊息點對點送給房內兩玩家（不走全域 broadcast）。

use std::net::SocketAddr;

use serde_json::{json, Value};
use tokio::time::{Duration, Instant};

use crate::games::chess::engine;
use crate::games::chess::hub::{Room, INCREMENT_MS, INITIAL_CLOCK_MS};
use crate::games::chess::types::{IllegalReason, Move, Side, Square, Status};
use crate::state::AppState;

/// 收到的 WS 文字訊息。回傳 true 表示已當作象棋訊息處理（呼叫端不再 echo）。
pub async fn handle(state: &AppState, who: SocketAddr, value: &Value) -> bool {
    match value.get("type").and_then(|v| v.as_str()) {
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
            handle_move(state, who, value.get("data")).await;
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

// ---- 配對 ----

async fn join_queue(state: &AppState, who: SocketAddr) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        // 防呆：對局中或已在佇列 → 忽略
        if hub.conn_room.contains_key(&who) || hub.queue.contains(&who) {
            return;
        }
        hub.queue.push_back(who);

        if hub.queue.len() >= 2 {
            let a = hub.queue.pop_front().unwrap();
            let b = hub.queue.pop_front().unwrap();
            // 隨機分紅黑（紅先行）
            let (red, black) = if rand::random::<bool>() { (a, b) } else { (b, a) };

            let id = hub.next_id;
            hub.next_id += 1;
            let room = Room {
                id,
                red,
                black,
                state: engine::initial_state(),
                red_ms: INITIAL_CLOCK_MS,
                black_ms: INITIAL_CLOCK_MS,
                turn_started_at: Instant::now(), // match_found 起算，紅先倒數
                ended: false,
            };
            hub.rooms.insert(id, room);
            hub.conn_room.insert(red, id);
            hub.conn_room.insert(black, id);

            outbox.push((
                red,
                msg("match_found", json!({ "color": "red", "clock_ms": INITIAL_CLOCK_MS })),
            ));
            outbox.push((
                black,
                msg("match_found", json!({ "color": "black", "clock_ms": INITIAL_CLOCK_MS })),
            ));
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

// ---- 行棋 ----

async fn handle_move(state: &AppState, who: SocketAddr, data: Option<&Value>) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;

        let Some(&room_id) = hub.conn_room.get(&who) else {
            flush(state, vec![(who, msg("error", json!({ "reason": "not_in_game" })))]);
            return;
        };
        let room = hub.rooms.get_mut(&room_id).unwrap();
        if room.ended {
            flush(state, vec![(who, msg("error", json!({ "reason": "game_ended" })))]);
            return;
        }

        let side = room.side_of(who).unwrap();
        if room.state.turn != side {
            let reason = IllegalReason::NotYourTurn.code();
            outbox.push((who, msg("illegal_move", json!({ "reason": reason }))));
            flush(state, outbox);
            return;
        }

        // 座標解析
        let (Some(from), Some(to)) = (
            parse_square(data.and_then(|d| d.get("from"))),
            parse_square(data.and_then(|d| d.get("to"))),
        ) else {
            outbox.push((who, msg("illegal_move", json!({ "reason": "bad_coord" }))));
            flush(state, outbox);
            return;
        };

        // 計時：先看行棋方是否已超時
        let now = Instant::now();
        let elapsed = now.duration_since(room.turn_started_at).as_millis() as i64;
        let remaining = room.clock_of(side) - elapsed;
        if remaining <= 0 {
            match side {
                Side::Red => room.red_ms = 0,
                Side::Black => room.black_ms = 0,
            }
            end_room(&mut hub, room_id, side.opponent(), "timeout", &mut outbox);
            flush(state, outbox);
            return;
        }

        // 合法性
        let mv = Move { from, to };
        if let Err(reason) = engine::is_legal(&room.state, mv) {
            outbox.push((who, msg("illegal_move", json!({ "reason": reason.code() }))));
            flush(state, outbox);
            return;
        }

        // 落子：扣時 + Fischer 增量，套用走步、切換回合
        let new_clock = remaining + INCREMENT_MS;
        match side {
            Side::Red => room.red_ms = new_clock,
            Side::Black => room.black_ms = new_clock,
        }
        room.turn_started_at = now;
        engine::apply(&mut room.state, mv);

        let move_payload = json!({
            "from": sq_json(from),
            "to": sq_json(to),
            "turn": room.state.turn.as_str(),
            "clock": { "red": room.red_ms, "black": room.black_ms },
        });
        let move_msg = msg("move_made", move_payload);
        outbox.push((room.red, move_msg.clone()));
        outbox.push((room.black, move_msg));

        // 結果判定
        match engine::game_status(&room.state) {
            Status::Ongoing => {
                // 將軍提示（被將方＝目前行棋方）
                if engine::is_in_check(&room.state, room.state.turn) {
                    let check_msg = msg("check", json!({ "side": room.state.turn.as_str() }));
                    outbox.push((room.red, check_msg.clone()));
                    outbox.push((room.black, check_msg));
                }
            }
            Status::Checkmate { winner } => {
                end_room(&mut hub, room_id, winner, "checkmate", &mut outbox);
            }
            Status::Stalemate { loser } => {
                end_room(&mut hub, room_id, loser.opponent(), "stalemate", &mut outbox);
            }
            Status::Draw => {
                end_room_draw(&mut hub, room_id, "draw_60", &mut outbox);
            }
        }
    }
    flush(state, outbox);
}

async fn resign(state: &AppState, who: SocketAddr) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        let Some(&room_id) = hub.conn_room.get(&who) else {
            return;
        };
        let Some(room) = hub.rooms.get(&room_id) else {
            return;
        };
        if room.ended {
            return;
        }
        let Some(side) = room.side_of(who) else {
            return;
        };
        end_room(&mut hub, room_id, side.opponent(), "resign", &mut outbox);
    }
    flush(state, outbox);
}

/// 連線斷開：在佇列就移除；在對局就判對手勝。
pub async fn handle_disconnect(state: &AppState, who: SocketAddr) {
    let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
    {
        let mut hub = state.chess().lock().await;
        hub.queue.retain(|&c| c != who);

        if let Some(&room_id) = hub.conn_room.get(&who) {
            if let Some(room) = hub.rooms.get(&room_id) {
                if !room.ended {
                    if let Some(side) = room.side_of(who) {
                        end_room(&mut hub, room_id, side.opponent(), "disconnect", &mut outbox);
                    }
                }
            }
        }
    }
    flush(state, outbox);
}

// ---- 房間結束 / 清理 ----

/// 推 game_over 給雙方並移除 room。`winner` 為勝方 side。
fn end_room(
    hub: &mut crate::games::chess::hub::ChessHubInner,
    room_id: u64,
    winner: Side,
    reason: &str,
    outbox: &mut Vec<(SocketAddr, String)>,
) {
    if let Some(room) = hub.rooms.get_mut(&room_id) {
        room.ended = true;
        let m = msg("game_over", json!({ "winner": winner.as_str(), "reason": reason }));
        outbox.push((room.red, m.clone()));
        outbox.push((room.black, m));
    }
    remove_room(hub, room_id);
}

/// 和棋（無勝方）。
fn end_room_draw(
    hub: &mut crate::games::chess::hub::ChessHubInner,
    room_id: u64,
    reason: &str,
    outbox: &mut Vec<(SocketAddr, String)>,
) {
    if let Some(room) = hub.rooms.get_mut(&room_id) {
        room.ended = true;
        let m = msg("game_over", json!({ "winner": Value::Null, "reason": reason }));
        outbox.push((room.red, m.clone()));
        outbox.push((room.black, m));
    }
    remove_room(hub, room_id);
}

fn remove_room(hub: &mut crate::games::chess::hub::ChessHubInner, room_id: u64) {
    if let Some(room) = hub.rooms.remove(&room_id) {
        hub.conn_room.remove(&room.red);
        hub.conn_room.remove(&room.black);
    }
}

fn flush(state: &AppState, outbox: Vec<(SocketAddr, String)>) {
    for (addr, m) in outbox {
        state.send_to(addr, m);
    }
}

// ---- 計時掃描：偵測行棋方時鐘耗盡卻無人走步 ----

/// 啟動時 spawn；每秒掃 rooms，當前行棋方超時即主動判負。
pub async fn timeout_watcher(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let mut outbox: Vec<(SocketAddr, String)> = Vec::new();
        {
            let mut hub = state.chess().lock().await;
            let now = Instant::now();
            let timed_out: Vec<(u64, Side)> = hub
                .rooms
                .values()
                .filter(|r| !r.ended)
                .filter_map(|r| {
                    let side = r.state.turn;
                    let elapsed = now.duration_since(r.turn_started_at).as_millis() as i64;
                    if r.clock_of(side) - elapsed <= 0 {
                        Some((r.id, side))
                    } else {
                        None
                    }
                })
                .collect();
            for (room_id, side) in timed_out {
                if let Some(room) = hub.rooms.get_mut(&room_id) {
                    match side {
                        Side::Red => room.red_ms = 0,
                        Side::Black => room.black_ms = 0,
                    }
                }
                end_room(&mut hub, room_id, side.opponent(), "timeout", &mut outbox);
            }
        }
        flush(&state, outbox);
    }
}

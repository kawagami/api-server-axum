//! 後台檔案直傳（手機 ↔ PC）的 WebRTC signaling。
//!
//! 檔案本身走瀏覽器之間的 DataChannel，**不經後端**；這裡只做兩件事：
//! - 房間：哪些連線開著傳輸頁（`TrackedConnection::transfer_ready`），有變動就推 `transfer_peers`
//!   給同帳號的所有在房連線
//! - 轉發：`transfer_offer` / `transfer_answer` / `transfer_reject` 原樣轉給 `data.to` 指定的連線，
//!   `to` 換成 `from`
//!
//! **只在同一個 admin 帳號（`user_name` 相同）的連線之間轉發** —— 匿名連線不能進房，
//! 也看不到、打不到別的帳號的裝置。admin 身分來自 ws ticket（已要求 `ws:read`），
//! 所以這裡不再另查權限。協定見 `protocol/transfer-wire.md`。

use crate::{
    state::AppState,
    structs::{features::Feature, ws::envelope},
};
use serde_json::{json, Value};
use std::net::SocketAddr;

/// 單則轉發訊息 `data` 的序列化上限。SDP（已含全部 ICE 候選，不做 trickle）約 2–5KB，
/// 加上檔案清單（檔名 + 大小）留足餘裕；超過的多半不是正常客戶端。
const MAX_RELAY_BYTES: usize = 64 * 1024;

/// 轉發時要驗證的連線最小視圖（純函式測試用，不必造出 `WsSender`）
#[derive(Clone, Copy)]
struct Member<'a> {
    user_name: Option<&'a str>,
    ready: bool,
}

/// 能否從 `from` 轉發給 `to`。`None` = 連線不存在。錯誤值即回給送出端的 `reason`。
fn check_relay(from: Option<Member>, to: Option<Member>, same_conn: bool) -> Result<(), &'static str> {
    let from = from.filter(|m| m.ready && m.user_name.is_some()).ok_or("not_joined")?;
    to.filter(|m| m.ready && !same_conn && m.user_name == from.user_name)
        .map(|_| ())
        .ok_or("peer_not_found")
}

/// 把上行 `data` 改寫成下行：拿掉 `to`、補上 `from`。`data` 不是物件時回 `None`。
fn rewrite_for_peer(data: &Value, from: SocketAddr) -> Option<Value> {
    let mut obj = data.as_object()?.clone();
    obj.remove("to");
    obj.insert("from".into(), Value::String(from.to_string()));
    Some(Value::Object(obj))
}

fn send_error(state: &AppState, who: SocketAddr, reason: &str) {
    state.send_to(who, envelope("transfer_error", json!({ "reason": reason })));
}

/// 沒帶 `game` 的上行訊息入口。不是 `transfer_*` 的一律忽略（與遊戲分派同一規則）。
pub(super) async fn dispatch_transfer(state: &AppState, who: SocketAddr, value: &Value) {
    let Some(typ) = value.get("type").and_then(|v| v.as_str()) else {
        return;
    };
    if !typ.starts_with("transfer_") {
        return;
    }
    if !state.get_settings().feature_enabled(Feature::FileTransfer) {
        send_error(state, who, "feature_disabled");
        return;
    }
    let data = value.get("data").unwrap_or(&Value::Null);
    match typ {
        "transfer_join" => set_ready(state, who, true).await,
        "transfer_leave" => set_ready(state, who, false).await,
        "transfer_offer" | "transfer_answer" | "transfer_reject" => relay(state, who, typ, data).await,
        _ => {}
    }
}

async fn set_ready(state: &AppState, who: SocketAddr, ready: bool) {
    let user_name = {
        let mut conns = state.get_connections().lock().await;
        let Some(conn) = conns.get_mut(&who) else { return };
        let Some(name) = conn.user_name.clone() else {
            drop(conns);
            send_error(state, who, "not_admin");
            return;
        };
        conn.transfer_ready = ready;
        name
    };
    push_peers(state, &user_name).await;
}

async fn relay(state: &AppState, who: SocketAddr, typ: &str, data: &Value) {
    if data.to_string().len() > MAX_RELAY_BYTES {
        send_error(state, who, "too_large");
        return;
    }
    let Some(to) = data
        .get("to")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<SocketAddr>().ok())
    else {
        send_error(state, who, "peer_not_found");
        return;
    };
    let Some(out) = rewrite_for_peer(data, who) else {
        return;
    };
    let checked = {
        let conns = state.get_connections().lock().await;
        let view = |addr: &SocketAddr| {
            conns.get(addr).map(|c| Member {
                user_name: c.user_name.as_deref(),
                ready: c.transfer_ready,
            })
        };
        check_relay(view(&who), view(&to), who == to)
    };
    match checked {
        Ok(()) => state.send_to(to, envelope(typ, out)),
        Err(reason) => send_error(state, who, reason),
    }
}

/// 推 `transfer_peers` 給 `user_name` 在房的每條連線，各自的清單不含自己。
/// 斷線清理（`socket::cleanup_connection`）也走這裡：連線已從 map 移除，自然不在清單裡。
pub(super) async fn push_peers(state: &AppState, user_name: &str) {
    let members: Vec<(SocketAddr, Value)> = {
        let conns = state.get_connections().lock().await;
        conns
            .iter()
            .filter(|(_, c)| c.transfer_ready && c.user_name.as_deref() == Some(user_name))
            .map(|(addr, c)| {
                (
                    *addr,
                    json!({
                        "addr": addr.to_string(),
                        "user_agent": c.user_agent,
                        "connected_at": super::connections::to_iso(c.connected_at),
                    }),
                )
            })
            .collect()
    };
    for (addr, _) in &members {
        let peers: Vec<&Value> = members
            .iter()
            .filter(|(other, _)| other != addr)
            .map(|(_, row)| row)
            .collect();
        state.send_to(*addr, envelope("transfer_peers", json!({ "peers": peers })));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(user_name: Option<&str>, ready: bool) -> Option<Member<'_>> {
        Some(Member { user_name, ready })
    }

    #[test]
    fn relay_between_same_account_ready_conns() {
        assert_eq!(check_relay(m(Some("kawa"), true), m(Some("kawa"), true), false), Ok(()));
    }

    #[test]
    fn sender_must_be_joined_admin() {
        assert_eq!(check_relay(m(None, true), m(Some("kawa"), true), false), Err("not_joined"));
        assert_eq!(check_relay(m(Some("kawa"), false), m(Some("kawa"), true), false), Err("not_joined"));
        assert_eq!(check_relay(None, m(Some("kawa"), true), false), Err("not_joined"));
    }

    #[test]
    fn target_must_be_other_ready_conn_of_same_account() {
        let from = m(Some("kawa"), true);
        assert_eq!(check_relay(from, m(Some("other"), true), false), Err("peer_not_found"));
        assert_eq!(check_relay(from, m(Some("kawa"), false), false), Err("peer_not_found"));
        assert_eq!(check_relay(from, m(None, true), false), Err("peer_not_found"));
        assert_eq!(check_relay(from, None, false), Err("peer_not_found"));
        assert_eq!(check_relay(from, from, true), Err("peer_not_found"));
    }

    #[test]
    fn rewrite_replaces_to_with_from() {
        let from: SocketAddr = "1.2.3.4:5678".parse().unwrap();
        let out = rewrite_for_peer(&json!({ "to": "9.9.9.9:1", "id": "x", "sdp": "v=0" }), from).unwrap();
        assert_eq!(out, json!({ "from": "1.2.3.4:5678", "id": "x", "sdp": "v=0" }));
        assert!(rewrite_for_peer(&json!("not an object"), from).is_none());
    }
}

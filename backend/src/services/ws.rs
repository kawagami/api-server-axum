//! WS 連線票（一次性 ticket）。
//!
//! JWT **不走** WS URL query —— query 會進 access log（`routes.rs` 的 request span），
//! 所以 admin 身分改用「登入中換一張 30 秒有效的 UUID 票 → 握手時一次性消費」。

use crate::{errors::AppError, repositories::redis, state::AppState};

/// 發一張新票（呼叫端須先確認有 `ws:read` 權限）。
///
/// ⚠️ **門檻必須與 `GET /ws/connections` 一致**：票換來的連線會被標成 admin 身分，
/// 因而收得到 `broadcast_to_admins` 的 `user_joined` / `user_left` —— 那兩則帶
/// `real_ip` / `user_email` / `user_agent`。
pub async fn issue_ticket(state: &AppState, user_name: &str) -> Result<String, AppError> {
    let ticket = uuid::Uuid::new_v4().to_string();
    redis::set_ws_ticket(state.get_redis_pool(), &ticket, user_name).await?;
    Ok(ticket)
}

/// 消費一張票，回傳票主。
///
/// 無票 / 票失效 / Redis 掛掉一律回 `None` = **匿名連線**（前台訪客即此路徑），
/// 不是錯誤 —— 握手不該因為身分認不出來就失敗。
pub async fn consume_ticket(state: &AppState, ticket: &str) -> Option<String> {
    redis::consume_ws_ticket(state.get_redis_pool(), ticket)
        .await
        .ok()
        .flatten()
}

// ---- 連線防護 ----
//
// `/ws` 是**匿名公開**端點，而 `middleware/rate_limit.rs` 只保護 HTTP：
// 握手前沒有任何額度檢查，握手後收訊迴圈也沒有。1 核 1G 的機器上這是兩個實際可用的
// 資源耗盡面：① 一個腳本開 N 條連線，每條都能在遊戲 hub 佔一張桌／一間房
// ② 單條連線用一個 while 迴圈灌訊息，每則都會去搶 hub 的 mutex 並廣播給大廳訂閱者。
// 下面兩道各擋一個，都是純記憶體判斷（不打 Redis，握手路徑不能再多一次 IO）。

use std::time::Instant;

/// 單一 IP 可同時保有的匿名 WS 連線數上限。
///
/// 取 12：同一個 NAT 出口（辦公室 / 校園 / 手機基地台）後面可能有好幾個人，
/// 每人再開幾個分頁都還在額度內；但擋掉「一個腳本開幾百條」。
/// **admin 身分（ticket 換來的連線）不受限** —— 後台自己就會開好幾個分頁，
/// 而且那條路徑已經要求 `ws:read` 權限。
pub const MAX_CONNECTIONS_PER_IP: usize = 12;

/// 目前該 IP 已有幾條連線。`connections` 只有幾十筆，線性掃比另外維護一份索引省事。
pub async fn ip_connection_count(state: &AppState, ip: &str) -> usize {
    let conns = state.get_connections().lock().await;
    conns.values().filter(|c| c.real_ip == ip).count()
}

/// 單條連線的收訊額度：容量 30、每秒回補 6。
///
/// 人手操作遠低於此（最快的連點也就每秒幾則），30 的容量容得下「進頁面同時送
/// join_lobby + 前一次暫存的訊息」這種正常突發；灌訊息的迴圈會在半秒內見底。
const BURST: f64 = 30.0;
const REFILL_PER_SEC: f64 = 6.0;

/// 令牌桶。每條連線一個，存在 socket task 的區域變數裡 —— 無共享狀態、無鎖。
pub struct MessageBudget {
    tokens: f64,
    last: Instant,
}

impl MessageBudget {
    pub fn new() -> Self {
        MessageBudget { tokens: BURST, last: Instant::now() }
    }

    /// 扣一格額度。回傳 false = 已超量（呼叫端負責收線）。
    pub fn try_consume(&mut self) -> bool {
        self.refill_at(Instant::now())
    }

    /// 測試用：可注入時間點。
    fn refill_at(&mut self, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        self.tokens = (self.tokens + elapsed * REFILL_PER_SEC).min(BURST);
        if self.tokens < 1.0 {
            return false;
        }
        self.tokens -= 1.0;
        true
    }
}

impl Default for MessageBudget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn burst_then_exhausted() {
        let mut b = MessageBudget::new();
        // 同一瞬間連送：容量內全過
        for i in 0..BURST as usize {
            assert!(b.try_consume(), "第 {i} 則應在容量內");
        }
        assert!(!b.try_consume(), "超過容量必須被擋");
    }

    #[test]
    fn refills_over_time() {
        let mut b = MessageBudget::new();
        let t0 = Instant::now();
        for _ in 0..BURST as usize {
            assert!(b.refill_at(t0));
        }
        assert!(!b.refill_at(t0));
        // 1 秒後回補 REFILL_PER_SEC 格
        let t1 = t0 + Duration::from_secs(1);
        for _ in 0..REFILL_PER_SEC as usize {
            assert!(b.refill_at(t1));
        }
        assert!(!b.refill_at(t1));
    }

    /// 長時間閒置不該累積出超過容量的額度（否則掛著一小時再一次爆發）
    #[test]
    fn refill_is_capped_at_burst() {
        let mut b = MessageBudget::new();
        let t0 = Instant::now();
        for _ in 0..BURST as usize {
            assert!(b.refill_at(t0));
        }
        let t1 = t0 + Duration::from_secs(3600);
        for _ in 0..BURST as usize {
            assert!(b.refill_at(t1));
        }
        assert!(!b.refill_at(t1), "閒置再久也只回補到容量上限");
    }
}

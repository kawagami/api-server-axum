use serde::{Deserialize, Serialize};

/// 中獎 email 通知的開關切換。
///
/// 發票對獎（`/member/invoices/notify`）與樂透對獎（`/member/lotto/notify`）是同一個
/// 動作、同一個形狀。收斂前 `structs/invoices.rs` 與 `structs/lotto.rs` 各一份，
/// 連 doc comment 都逐字相同 —— 新增第三種通知偏好時直接用這組，不要再長一份。
#[derive(Deserialize)]
pub struct NotifyPrefRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct NotifyPrefResponse {
    pub enabled: bool,
}

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 一張登錄的發票（DB 對應）
#[derive(Clone, Serialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub member_id: i64,
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    pub period: String,
    pub amount: Option<Decimal>,
    pub seller_tax_id: Option<String>,
    pub source: String, // 'qr' | 'barcode' | 'manual'
    pub ledger_entry_id: Option<Uuid>,
    pub lottery_checked: bool,
    pub prize_tier: Option<String>,
    pub notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 登錄發票請求（QR / barcode / manual 共用前門）
#[derive(Deserialize)]
pub struct InvoiceRequest {
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    pub amount: Option<Decimal>,
    pub seller_tax_id: Option<String>,
    pub source: String,
    #[serde(default)]
    pub record_as_expense: bool, // true 時一併建 ledger expense 並連結
    pub category: Option<String>,
    pub note: Option<String>,
}

/// 列表查詢
#[derive(Deserialize)]
pub struct InvoiceListQuery {
    pub period: Option<String>,
    pub won: Option<bool>, // true=只看中獎、false=只看未中
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 開獎號碼查詢
#[derive(Deserialize)]
pub struct DrawListQuery {
    pub period: Option<String>, // 指定期別；省略則回近期各期
    pub limit: Option<i64>,     // 回傳期數（預設 6、上限 24）
}

/// 某一期的中獎號碼（前端展示用，一期一筆）
#[derive(Serialize)]
pub struct PeriodDraw {
    pub period: String,
    pub special: Option<String>, // 特別獎（8 碼）
    pub grand: Option<String>,   // 特獎（8 碼）
    pub first: Vec<String>,      // 頭獎（8 碼，通常 3 組）
    pub additional: Vec<String>, // 增開六獎（3 碼，0~N 組）
}

/// 通知偏好切換
#[derive(Deserialize)]
pub struct NotifyPrefRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct NotifyPrefResponse {
    pub enabled: bool,
}

/// admin 手動補中獎號碼
#[derive(Deserialize)]
pub struct AdminLotteryNumbersRequest {
    pub period: String,
    pub special: Option<String>,
    pub grand: Option<String>,
    #[serde(default)]
    pub first: Vec<String>,
    #[serde(default)]
    pub additional: Vec<String>,
}

/// 對獎 job 寄信用：一筆中獎發票 + 收件 member email
#[derive(FromRow)]
pub struct WinnerRow {
    pub id: Uuid,
    pub member_id: i64,
    pub invoice_number: String,
    pub period: String,
    pub prize_tier: String,
    pub email: String,
}

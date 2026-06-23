use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 全站固定分類：(value, 中文 label)。前端下拉選單以此為準。
pub const EXPENSE_CATEGORIES: &[(&str, &str)] = &[
    ("food", "餐飲"),
    ("transport", "交通"),
    ("housing", "居住"),
    ("utilities", "水電瓦斯"),
    ("daily", "日用"),
    ("entertainment", "娛樂"),
    ("medical", "醫療"),
    ("education", "教育"),
    ("social", "人情"),
    ("other", "其他"),
];

pub const INCOME_CATEGORIES: &[(&str, &str)] = &[
    ("salary", "薪資"),
    ("bonus", "獎金"),
    ("investment", "投資"),
    ("part_time", "兼職"),
    ("other", "其他"),
];

/// 一筆記帳資料（DB 對應）
#[derive(Clone, Serialize, FromRow)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub member_id: i64,
    pub kind: String,
    pub amount: Decimal,
    pub category: String,
    pub note: Option<String>,
    pub occurred_at: NaiveDate,
    pub invoice_number: Option<String>, // 發票號碼（手動建立為 null）
    pub seller_tax_id: Option<String>,  // 賣方統編
    pub source: String,                 // 'manual' | 'invoice_qr'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 新增 / 更新請求 body（手動記帳）
#[derive(Deserialize)]
pub struct LedgerRequest {
    pub kind: String,
    pub amount: Decimal,
    pub category: String,
    pub note: Option<String>,
    pub occurred_at: NaiveDate,
}

/// 列表查詢參數：分頁 + kind / category / 日期區間 filter
#[derive(Deserialize)]
pub struct LedgerListQuery {
    pub kind: Option<String>,
    pub category: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// summary 查詢參數：統計區間（不帶則涵蓋全部）
#[derive(Deserialize)]
pub struct SummaryQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

/// 統計總覽：總收入 / 總支出 / 結餘 + 分類加總 + 每月趨勢
#[derive(Serialize)]
pub struct LedgerSummary {
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub balance: Decimal,
    pub by_category: Vec<CategorySum>,
    pub monthly: Vec<MonthlySum>,
}

#[derive(Serialize, FromRow)]
pub struct CategorySum {
    pub kind: String,
    pub category: String,
    pub total: Decimal,
}

#[derive(Serialize, FromRow)]
pub struct MonthlySum {
    pub month: String, // 'YYYY-MM'
    pub income: Decimal,
    pub expense: Decimal,
}

/// GET /categories 回傳結構
#[derive(Serialize)]
pub struct CategoryOption {
    pub value: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct CategoryList {
    pub income: Vec<CategoryOption>,
    pub expense: Vec<CategoryOption>,
}

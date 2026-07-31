use serde::Deserialize;

pub const MAX_PER_PAGE: i64 = 200;

/// 共用分頁參數：?page=1&per_page=50（page 從 1 起算）
///
/// ⚠️ **不要試圖用 `#[serde(flatten)] page: PageQuery` 併進別的 query struct**。
/// `axum::extract::Query` 走 `serde_urlencoded`，flatten 會讓 serde 先把值緩衝成字串、
/// 之後無法轉回 `i64`，runtime 直接噴 `invalid type: string "3", expected i64`
/// —— 而且**編譯期完全看不出來**（2026-07-31 實測過）。
/// 所以 `LedgerListQuery` / `TicketListQuery` / `InvoiceListQuery` 是刻意各自重複宣告
/// `page` / `per_page`，再手動組成 `PageQuery` 呼叫 `to_limit_offset`。
/// 真要收斂得換成 `axum_extra::extract::Query`（走 `serde_html_form`，支援 flatten），
/// 但那會讓少數端點用不同的 extractor，不划算。
#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PageQuery {
    /// 轉成 SQL 用的 (limit, offset)；per_page 夾在 1..=MAX_PER_PAGE
    pub fn to_limit_offset(&self, default_per_page: i64) -> (i64, i64) {
        let per_page = self
            .per_page
            .unwrap_or(default_per_page)
            .clamp(1, MAX_PER_PAGE);
        let page = self.page.unwrap_or(1).max(1);
        (per_page, (page - 1) * per_page)
    }
}

use serde::{Deserialize, Serialize};

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

/// 共用的 `?status=` 篩選參數。
///
/// 與 `PageQuery` 同一種東西（跨資源共用的 query 參數），所以放同一個模組。
/// 收斂前 `routes/stocks.rs` 與 `routes/torrents.rs` 各宣告一份逐字相同的私有版本。
/// 值本身不在這裡驗 —— 各資源的合法狀態集不同，由 repository 的 WHERE 決定。
#[derive(Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

/// 分頁回應的共用形狀 `{ data, total }`。
///
/// `total` 是**套用篩選後、未套 limit/offset 的總筆數**，前端靠它算頁碼。
/// 全站另有 5 個逐字同形的 `XxxPaginatedResponse`（torrents / gov_tenders / stocks /
/// messages / blog_comments）待收斂進來 —— **新端點一律用這個，不要再長第 6 個**。
#[derive(Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub total: i64,
}

impl<T> Paginated<T> {
    pub fn new(data: Vec<T>, total: i64) -> Self {
        Self { data, total }
    }
}

#[cfg(test)]
mod tests {
    use super::{PageQuery, MAX_PER_PAGE};

    fn query(page: Option<i64>, per_page: Option<i64>) -> PageQuery {
        PageQuery { page, per_page }
    }

    #[test]
    fn defaults_when_params_absent() {
        assert_eq!(query(None, None).to_limit_offset(50), (50, 0));
    }

    #[test]
    fn offset_counts_from_page_one() {
        assert_eq!(query(Some(1), Some(20)).to_limit_offset(50), (20, 0));
        assert_eq!(query(Some(3), Some(20)).to_limit_offset(50), (20, 40));
    }

    /// per_page 夾在 1..=MAX_PER_PAGE：0 / 負數 / 超上限都不該漏出去變成 SQL 的 LIMIT
    #[test]
    fn per_page_is_clamped() {
        assert_eq!(query(None, Some(0)).to_limit_offset(50), (1, 0));
        assert_eq!(query(None, Some(-10)).to_limit_offset(50), (1, 0));
        assert_eq!(
            query(None, Some(MAX_PER_PAGE + 1)).to_limit_offset(50),
            (MAX_PER_PAGE, 0)
        );
    }

    /// page 0 與負數一律當第 1 頁，offset 不可為負（負 offset 是 SQL 錯誤）
    #[test]
    fn page_below_one_falls_back_to_first_page() {
        assert_eq!(query(Some(0), Some(20)).to_limit_offset(50), (20, 0));
        assert_eq!(query(Some(-5), Some(20)).to_limit_offset(50), (20, 0));
    }
}

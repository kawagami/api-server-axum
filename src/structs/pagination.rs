use serde::Deserialize;

pub const MAX_PER_PAGE: i64 = 200;

/// 共用分頁參數：?page=1&per_page=50（page 從 1 起算）
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

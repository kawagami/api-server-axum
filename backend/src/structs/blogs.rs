use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// `PUT /admin/blogs/:id` 的 body。
///
/// **不收 `tocs`** —— 標題由後端從 `markdown` 解析（`services::blogs::extract_toc_texts`）。
/// 以前是 client 自己 parse 一份送上來，等於讓呼叫端決定文章標題。
#[derive(Deserialize, Serialize)]
pub struct PutBlog {
    pub markdown: String,
    pub tags: Vec<String>,
}

/// blogs 公開列表的過濾條件（分頁走共用 `PageQuery`）。tag / author 可各自獨立或並用。
#[derive(Deserialize)]
pub struct BlogFilter {
    pub tag: Option<String>,
    /// 作者頁用：只列此 admin（users.name）的文章
    pub author: Option<String>,
    /// 關鍵字：對 markdown 內容 ILIKE 模糊比對（含標題，因標題也在 markdown 內）
    pub q: Option<String>,
    /// 排序：`oldest` = 建立時間舊→新；其餘（含省略）= 新→舊
    pub sort: Option<String>,
}

/// 後台列表的過濾條件（分頁走共用 `PageQuery`）。
///
/// 刻意不重用公開的 `BlogFilter`：後台**沒有 `author`** —— 擁有者由 session 決定
/// （`auth_user.owner_filter()`），若也收 query 參數，一般 admin 帶 `?author=別人`
/// 就能列出不屬於自己的文章。
#[derive(Deserialize)]
pub struct AdminBlogFilter {
    pub tag: Option<String>,
    /// 關鍵字：對 markdown 內容 ILIKE 模糊比對（含標題，因標題也在 markdown 內）
    pub q: Option<String>,
    /// 排序，見 `AdminBlogSort::from_query`
    pub sort: Option<String>,
}

/// 後台列表排序。
///
/// `ORDER BY` 不能 bind、只能字串內插，所以不讓 query 字串一路走到 SQL：
/// 在這裡收斂成列舉（未知值一律 fallback `Newest`），repository 只吃列舉。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AdminBlogSort {
    Newest,
    Oldest,
    /// 後台最常問的問題是「哪篇最近改過」，公開列表沒有這個排序
    RecentlyUpdated,
}

impl AdminBlogSort {
    pub fn from_query(sort: Option<&str>) -> Self {
        match sort {
            Some("oldest") => Self::Oldest,
            Some("updated") => Self::RecentlyUpdated,
            _ => Self::Newest,
        }
    }

    /// 直接內插進 SQL 的片段 —— 只有這裡產生，且來源是封閉列舉
    pub fn order_by(self) -> &'static str {
        match self {
            Self::Newest => "created_at DESC",
            Self::Oldest => "created_at ASC",
            Self::RecentlyUpdated => "updated_at DESC",
        }
    }
}

/// 後台列表的一列。
///
/// **刻意不含 `markdown`** —— 清單只顯示 `tocs[0]`、tags 與時間，但 `DbBlog` 帶全文，
/// 一頁 50 篇就等於把那 50 篇的完整內容塞進 SSR payload（1 核 1G 的 VPS 上很有感）。
#[derive(Serialize, FromRow)]
pub struct AdminBlogListItem {
    pub id: Uuid,
    pub tocs: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// tag 與其文章數（公開列表側欄用）
#[derive(Serialize, FromRow)]
pub struct TagCount {
    pub tag: String,
    pub count: i64,
}

/// 後台改名/合併 tag 請求
#[derive(Deserialize)]
pub struct RenameTagRequest {
    pub from: String,
    pub to: String,
}

/// 後台全站刪除 tag 的查詢參數
#[derive(Deserialize)]
pub struct DeleteTagQuery {
    pub tag: String,
}

/// tag 變更結果：回受影響文章數
#[derive(Serialize)]
pub struct TagMutationResponse {
    pub affected: u64,
}

#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct DbBlog {
    pub id: Uuid,
    pub markdown: String,
    pub tocs: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 作者（admin）顯示名；公開列表/內文會 JOIN users 帶出，其餘查詢預設 None
    #[sqlx(default)]
    pub author_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_query_falls_back_to_newest() {
        assert_eq!(AdminBlogSort::from_query(None), AdminBlogSort::Newest);
        assert_eq!(AdminBlogSort::from_query(Some("")), AdminBlogSort::Newest);
        assert_eq!(AdminBlogSort::from_query(Some("created_at; DROP TABLE blogs")), AdminBlogSort::Newest);
    }

    #[test]
    fn sort_query_maps_known_values() {
        assert_eq!(AdminBlogSort::from_query(Some("oldest")), AdminBlogSort::Oldest);
        assert_eq!(AdminBlogSort::from_query(Some("updated")), AdminBlogSort::RecentlyUpdated);
    }
}

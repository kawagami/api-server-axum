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

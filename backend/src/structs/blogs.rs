use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct PutBlog {
    pub markdown: String,
    pub tocs: Vec<Toc>,
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

impl PutBlog {
    /// 提取 tocs 中的 text 字段，返回 Vec<String>
    pub fn extract_toc_texts(&self) -> Vec<String> {
        self.tocs.iter().map(|toc| toc.text.clone()).collect()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Toc {
    id: String,
    level: u32,
    text: String,
}

#[derive(Serialize)]
pub struct BlogsResponse {
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
    pub data: Vec<DbBlog>,
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

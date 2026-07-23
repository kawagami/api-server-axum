use crate::{
    errors::{AppError, RequestError},
    repositories::blog_comments as repo,
    structs::blog_comments::{BlogComment, BlogCommentPaginatedResponse, NewComment},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

const CONTENT_MAX: usize = 5000;
const NAME_MAX: usize = 100;

/// 修剪字串,空字串視為 None(訪客名選填,可匿名)
fn normalize_optional(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// 驗證並建立一則留言。
/// `member_id`:optional-auth 中介層帶入,有值 = 會員留言(忽略自填名),null = 訪客留言。
/// content 必填非空、長度上限;訪客名選填但有長度上限。違規回 422,blog 不存在回 404。
pub async fn create(
    pool: &Pool<Postgres>,
    blog_id: Uuid,
    member_id: Option<i64>,
    input: NewComment,
) -> Result<BlogComment, AppError> {
    let content = input.content.trim();
    if content.is_empty() {
        return Err(RequestError::UnprocessableContent("留言內容不可為空".into()).into());
    }
    if content.chars().count() > CONTENT_MAX {
        return Err(
            RequestError::UnprocessableContent(format!("留言內容不可超過 {CONTENT_MAX} 字")).into(),
        );
    }

    if !repo::blog_exists(pool, blog_id).await? {
        return Err(RequestError::NotFound.into());
    }

    // 會員留言的顯示名/頭像由 member_id join 取得,訪客才用自填名
    let author_name = if member_id.is_some() {
        None
    } else {
        let name = normalize_optional(input.name);
        if let Some(n) = &name {
            if n.chars().count() > NAME_MAX {
                return Err(
                    RequestError::UnprocessableContent(format!("名字不可超過 {NAME_MAX} 字")).into(),
                );
            }
        }
        name
    };

    repo::insert(pool, blog_id, member_id, author_name.as_deref(), content).await
}

/// 單篇 blog 的公開留言分頁
pub async fn list_by_blog(
    pool: &Pool<Postgres>,
    blog_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<BlogCommentPaginatedResponse, AppError> {
    let total = repo::count_by_blog(pool, blog_id).await?;
    let data = repo::list_by_blog(pool, blog_id, limit, offset).await?;
    Ok(BlogCommentPaginatedResponse { data, total })
}

/// 後台:全站留言分頁
pub async fn list_all(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<BlogCommentPaginatedResponse, AppError> {
    let total = repo::count_all(pool).await?;
    let data = repo::list_all(pool, limit, offset).await?;
    Ok(BlogCommentPaginatedResponse { data, total })
}

pub async fn delete(pool: &Pool<Postgres>, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}

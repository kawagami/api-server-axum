use crate::{
    errors::{AppError, RequestError},
    repositories::blog_comments as repo,
    structs::auth::AuthenticatedUser,
    structs::pagination::Paginated,
    structs::blog_comments::{BlogComment, NewComment},
    utils::text::normalize_optional,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

const CONTENT_MAX: usize = 5000;
const NAME_MAX: usize = 100;

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
) -> Result<Paginated<BlogComment>, AppError> {
    // count 與 list 併發跑：序列 await 是白吃一倍延遲（範本同 services/logs.rs）
    let (data, total) = tokio::try_join!(
        repo::list_by_blog(pool, blog_id, limit, offset),
        repo::count_by_blog(pool, blog_id),
    )?;
    Ok(Paginated::new(data, total))
}

/// 後台:全站留言分頁
pub async fn list_all(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<Paginated<BlogComment>, AppError> {
    let (data, total) = tokio::try_join!(repo::list_all(pool, limit, offset), repo::count_all(pool))?;
    Ok(Paginated::new(data, total))
}

/// 刪一則留言。
///
/// **與 blog 本體同一套 owner 隔離**：一般管理員只能刪自己文章下的留言。少了這道，
/// blog 本體嚴格隔離、留言卻不設限，等於留了一個側門 —— 所以檢查跟刪除綁在同一支。
pub async fn delete(pool: &Pool<Postgres>, actor: &AuthenticatedUser, id: i64) -> Result<(), AppError> {
    let author = repo::get_blog_author(pool, id)
        .await?
        .ok_or(RequestError::NotFound)?;
    actor.require_owner(author)?;
    repo::delete(pool, id).await
}

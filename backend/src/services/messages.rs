use crate::{
    errors::{AppError, RequestError},
    repositories::messages as repo,
    structs::messages::{Message, MessagePaginatedResponse, NewMessage},
};
use sqlx::{Pool, Postgres};

const CONTENT_MAX: usize = 5000;
const NAME_MAX: usize = 100;
const EMAIL_MAX: usize = 200;

/// 修剪字串,空字串視為 None(name / email 選填,可匿名留言)
fn normalize_optional(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// 驗證並建立一則留言。content 必填非空、長度上限;name / email 選填但有長度上限,
/// email 若有值做最基本格式檢查(含 `@`、無空白)。違規回 422。
pub async fn create(pool: &Pool<Postgres>, input: NewMessage) -> Result<Message, AppError> {
    let content = input.content.trim();
    if content.is_empty() {
        return Err(RequestError::UnprocessableContent("留言內容不可為空".into()).into());
    }
    if content.chars().count() > CONTENT_MAX {
        return Err(
            RequestError::UnprocessableContent(format!("留言內容不可超過 {CONTENT_MAX} 字")).into(),
        );
    }

    let name = normalize_optional(input.name);
    if let Some(n) = &name {
        if n.chars().count() > NAME_MAX {
            return Err(
                RequestError::UnprocessableContent(format!("名字不可超過 {NAME_MAX} 字")).into(),
            );
        }
    }

    let email = normalize_optional(input.email);
    if let Some(e) = &email {
        if e.chars().count() > EMAIL_MAX
            || !e.contains('@')
            || e.chars().any(|c| c.is_whitespace())
        {
            return Err(RequestError::UnprocessableContent("email 格式不正確".into()).into());
        }
    }

    repo::insert(pool, name.as_deref(), email.as_deref(), content).await
}

pub async fn list(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<MessagePaginatedResponse, AppError> {
    let total = repo::count(pool).await?;
    let data = repo::list(pool, limit, offset).await?;
    Ok(MessagePaginatedResponse { data, total })
}

pub async fn delete(pool: &Pool<Postgres>, id: i64) -> Result<(), AppError> {
    repo::delete(pool, id).await
}

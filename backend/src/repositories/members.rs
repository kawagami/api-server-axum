use crate::{
    errors::AppError,
    structs::members::{Member, MemberDetail},
};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

/// 會員列表。全站每個列表端點都吃 PageQuery，這支原本是唯一沒有 LIMIT 的
/// （而且回傳含 email），members 表只會單向成長。
pub async fn get_members(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Member>, AppError> {
    let members = sqlx::query_as(
        "SELECT id, name, email, avatar_url, created_at FROM members
         ORDER BY id DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(members)
}

pub async fn count_members(pool: &Pool<Postgres>) -> Result<i64, AppError> {
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM members")
        .fetch_one(pool)
        .await?;
    Ok(total)
}

/// 會員明細。
///
/// **兩支查詢，併發跑**。收斂前是三支序列：`members` 查一次拿基本欄位、`member_oauth`
/// 查一次、然後**再查一次 `members` 同一列**只為了兩個 bool 欄位。同一張表同一列查兩次
/// 是純粹的浪費，而 `member_oauth` 那支不依賴前者（會員不存在時它本來就回空陣列），
/// 所以序列等待也是白吃的延遲。
pub async fn get_member_by_id(pool: &Pool<Postgres>, id: i64) -> Result<Option<MemberDetail>, AppError> {
    let member = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, DateTime<Utc>, bool, bool)>(
        "SELECT id, name, email, avatar_url, created_at,
                lottery_notify_enabled, lotto_notify_enabled
         FROM members WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool);

    let providers = sqlx::query_scalar::<_, String>(
        "SELECT provider FROM member_oauth WHERE member_id = $1",
    )
    .bind(id)
    .fetch_all(pool);

    let (member, providers) = tokio::try_join!(member, providers)?;

    let Some((id, name, email, avatar_url, created_at, lottery_notify_enabled, lotto_notify_enabled)) =
        member
    else {
        return Ok(None);
    };

    Ok(Some(MemberDetail {
        id,
        name,
        email,
        avatar_url,
        created_at,
        providers,
        lottery_notify_enabled,
        lotto_notify_enabled,
    }))
}

pub async fn find_or_create_by_oauth(
    pool: &Pool<Postgres>,
    provider: &str,
    provider_id: &str,
    name: &str,
    email: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<i64, AppError> {
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT member_id FROM member_oauth WHERE provider = $1 AND provider_id = $2",
    )
    .bind(provider)
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    if let Some((member_id,)) = existing {
        return Ok(member_id);
    }

    let mut tx = pool.begin().await?;

    let (member_id,): (i64,) = sqlx::query_as(
        "INSERT INTO members (name, email, avatar_url) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(email)
    .bind(avatar_url)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO member_oauth (member_id, provider, provider_id) VALUES ($1, $2, $3)",
    )
    .bind(member_id)
    .bind(provider)
    .bind(provider_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(member_id)
}

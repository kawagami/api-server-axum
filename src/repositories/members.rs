use crate::{errors::AppError, state::AppState};

pub async fn find_or_create_by_oauth(
    state: &AppState,
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
    .fetch_optional(state.get_pool())
    .await?;

    if let Some((member_id,)) = existing {
        return Ok(member_id);
    }

    let mut tx = state.get_pool().begin().await?;

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

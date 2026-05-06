use crate::{
    errors::AppError,
    state::AppState,
    structs::members::{Member, MemberDetail},
};

pub async fn get_members(state: &AppState) -> Result<Vec<Member>, AppError> {
    let members = sqlx::query_as(
        "SELECT id, name, email, avatar_url, created_at FROM members ORDER BY id DESC",
    )
    .fetch_all(state.get_pool())
    .await?;
    Ok(members)
}

pub async fn get_member_by_id(state: &AppState, id: i64) -> Result<Option<MemberDetail>, AppError> {
    let member: Option<Member> = sqlx::query_as(
        "SELECT id, name, email, avatar_url, created_at FROM members WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(state.get_pool())
    .await?;

    let Some(member) = member else { return Ok(None) };

    let providers: Vec<String> =
        sqlx::query_scalar("SELECT provider FROM member_oauth WHERE member_id = $1")
            .bind(id)
            .fetch_all(state.get_pool())
            .await?;

    Ok(Some(MemberDetail {
        id: member.id,
        name: member.name,
        email: member.email,
        avatar_url: member.avatar_url,
        created_at: member.created_at,
        providers,
    }))
}

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

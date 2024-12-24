use crate::{
    state::AppStateV2,
    structs::users::{DbUser, User},
};

pub async fn get_users(state: &AppStateV2) -> Result<Vec<User>, sqlx::Error> {
    let pool = state.get_pool();

    sqlx::query_as("SELECT id, name, email FROM users")
        .fetch_all(&pool)
        .await
}

pub async fn check_email_exists(state: &AppStateV2, email: &str) -> Result<DbUser, sqlx::Error> {
    // 使用 EXISTS 查詢是否有特定 email
    let result: DbUser = sqlx::query_as(
        r#"
            SELECT
                id,
                email,
                password
            FROM
                users
            WHERE
                email = $1
            LIMIT
                1;
        "#,
    )
    .bind(email)
    .fetch_one(&state.get_pool())
    .await?;

    Ok(result)
}

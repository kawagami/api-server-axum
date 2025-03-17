use crate::{
    state::AppStateV2,
    structs::users::{DbUser, NewUser, User},
};
use sqlx::Error;

pub async fn get_users(state: &AppStateV2) -> Result<Vec<User>, Error> {
    sqlx::query_as("SELECT id, name, email FROM users")
        .fetch_all(state.get_pool())
        .await
}

pub async fn check_email_exists(state: &AppStateV2, email: &str) -> Result<DbUser, Error> {
    sqlx::query_as(
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
    .fetch_one(state.get_pool())
    .await
}

pub async fn create_user(state: &AppStateV2, new_user: NewUser) -> Result<(), Error> {
    let _ = sqlx::query(
        r#"
            INSERT INTO users (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING id, name, email;
        "#,
    )
    .bind(&new_user.name)
    .bind(&new_user.email)
    .bind(&new_user.password)
    .execute(state.get_pool())
    .await?;

    Ok(())
}

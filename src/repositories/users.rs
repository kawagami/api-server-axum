use crate::{state::AppStateV2, structs::users::User};

pub async fn get_users(state: &AppStateV2) -> Result<Vec<User>, sqlx::Error> {
    let pool = state.get_pool();

    sqlx::query_as("SELECT id, name, email FROM users")
        .fetch_all(&pool)
        .await
}

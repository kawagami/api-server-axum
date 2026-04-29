use crate::{
    errors::AppError,
    repositories::users as users_repo,
    state::AppStateV2,
    structs::users::{NewUser, User},
};

pub async fn get_users(state: &AppStateV2) -> Result<Vec<User>, AppError> {
    users_repo::get_users(state).await
}

pub async fn create_user(state: &AppStateV2, user: NewUser) -> Result<(), AppError> {
    users_repo::create_user(state, user).await
}

use crate::{
    errors::AppError,
    repositories::app_settings as repo,
    state::AppState,
    structs::app_settings::AppSetting,
};

pub async fn get_all(state: &AppState) -> Result<Vec<AppSetting>, AppError> {
    repo::get_all(state.get_pool()).await
}

pub async fn update(state: &AppState, key: &str, value: &str) -> Result<AppSetting, AppError> {
    let setting = repo::update(state.get_pool(), key, value).await?;
    state.reload_settings().await;
    Ok(setting)
}

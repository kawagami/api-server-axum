use crate::{
    errors::AppError,
    repositories::app_settings as repo,
    state::AppState,
    structs::app_settings::AppSetting,
};
use std::collections::BTreeMap;

pub async fn get_all(state: &AppState) -> Result<BTreeMap<String, Vec<AppSetting>>, AppError> {
    let rows = repo::get_all(state.get_pool()).await?;
    let mut grouped: BTreeMap<String, Vec<AppSetting>> = BTreeMap::new();
    for setting in rows {
        grouped.entry(setting.category.clone()).or_default().push(setting);
    }
    Ok(grouped)
}

pub async fn update(state: &AppState, key: &str, value: &str) -> Result<AppSetting, AppError> {
    let setting = repo::update(state.get_pool(), key, value).await?;
    state.reload_settings().await;
    Ok(setting)
}

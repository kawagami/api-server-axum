use crate::{
    errors::AppError,
    repositories::app_settings as repo,
    state::Settings,
    structs::app_settings::AppSetting,
};
use sqlx::{Pool, Postgres};
use std::collections::BTreeMap;

pub async fn get_all(pool: &Pool<Postgres>) -> Result<BTreeMap<String, Vec<AppSetting>>, AppError> {
    let rows = repo::get_all(pool).await?;
    let mut grouped: BTreeMap<String, Vec<AppSetting>> = BTreeMap::new();
    for setting in rows {
        grouped.entry(setting.category.clone()).or_default().push(setting);
    }
    Ok(grouped)
}

pub async fn update(
    pool: &Pool<Postgres>,
    settings: &Settings,
    key: &str,
    value: &str,
) -> Result<AppSetting, AppError> {
    let setting = repo::update(pool, key, value).await?;
    settings.reload(pool).await;
    Ok(setting)
}

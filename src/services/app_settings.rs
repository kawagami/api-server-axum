use crate::{
    errors::{AppError, RequestError},
    repositories::app_settings as repo,
    state::Settings,
    structs::app_settings::AppSetting,
};
use sqlx::{Pool, Postgres};
use std::collections::BTreeMap;

/// 可由無認證端點讀取的設定白名單 — 新增公開設定時在此加 key
const PUBLIC_KEYS: &[&str] = &["site_theme", "default_color_mode"];

/// 設定值驗證 — key 不在表內就不驗證
fn validate(key: &str, value: &str) -> Result<(), AppError> {
    let allowed: &[&str] = match key {
        "site_theme" => &["forest", "ocean", "sky"],
        "default_color_mode" => &["light", "dark", "system"],
        _ => return Ok(()),
    };
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(RequestError::UnprocessableContent(format!(
            "{} 只接受 {}",
            key,
            allowed.join(" / ")
        ))
        .into())
    }
}

/// 公開設定 — 直接讀記憶體中的 settings map（PATCH 時已自動 reload），不打 DB
pub fn get_public(settings: &Settings) -> BTreeMap<String, String> {
    PUBLIC_KEYS
        .iter()
        .filter_map(|key| settings.get(key).map(|v| (key.to_string(), v)))
        .collect()
}

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
    validate(key, value)?;
    let setting = repo::update(pool, key, value).await?;
    settings.reload(pool).await;
    Ok(setting)
}

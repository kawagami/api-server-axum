use crate::{
    errors::{AppError, RequestError},
    repositories::app_settings as repo,
    state::Settings,
    structs::app_settings::AppSetting,
};
use sqlx::{Pool, Postgres};
use std::collections::BTreeMap;

/// 可由無認證端點讀取的設定白名單 — 新增公開設定時在此加 key
const PUBLIC_KEYS: &[&str] = &["site_theme", "default_color_mode", "theme_rotation"];

/// 全部主題清單 — 與前端 libs/site-theme.ts 的 SITE_THEMES 一致
const SITE_THEMES: &[&str] = &["forest", "ocean", "sky", "sunset", "sakura", "grape", "mono"];

fn unprocessable(msg: String) -> AppError {
    RequestError::UnprocessableContent(msg).into()
}

/// theme_rotation 驗證：JSON 物件，key 剛好 "0".."6"，value 為 SITE_THEMES 之一（拒 auto）
fn validate_theme_rotation(value: &str) -> Result<(), AppError> {
    let map: std::collections::HashMap<String, String> = serde_json::from_str(value)
        .map_err(|_| unprocessable("theme_rotation 必須是合法 JSON 物件".into()))?;

    let expected: std::collections::HashSet<&str> =
        ["0", "1", "2", "3", "4", "5", "6"].into_iter().collect();
    let got: std::collections::HashSet<&str> = map.keys().map(String::as_str).collect();
    if got != expected {
        return Err(unprocessable("theme_rotation 的 key 必須剛好為 \"0\"–\"6\"".into()));
    }

    for v in map.values() {
        if !SITE_THEMES.contains(&v.as_str()) {
            return Err(unprocessable(format!(
                "theme_rotation 主題只接受 {}",
                SITE_THEMES.join(" / ")
            )));
        }
    }
    Ok(())
}

/// 設定值驗證 — key 不在表內就不驗證
fn validate(key: &str, value: &str) -> Result<(), AppError> {
    if key == "theme_rotation" {
        return validate_theme_rotation(value);
    }

    let allowed: Vec<&str> = match key {
        // site_theme = 7 套主題 ＋ auto（auto = 走每日輪播）
        "site_theme" => SITE_THEMES.iter().copied().chain(std::iter::once("auto")).collect(),
        "default_color_mode" => vec!["light", "dark", "system"],
        _ => return Ok(()),
    };
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(unprocessable(format!("{} 只接受 {}", key, allowed.join(" / "))))
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

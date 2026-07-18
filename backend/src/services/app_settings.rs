use crate::{
    errors::{AppError, RequestError},
    repositories::app_settings as repo,
    state::Settings,
    structs::{app_settings::AppSetting, features::Feature},
};
use sqlx::{Pool, Postgres};
use std::collections::BTreeMap;

/// 可由無認證端點讀取的設定白名單 — 新增公開設定時在此加 key
const PUBLIC_KEYS: &[&str] = &[
    "site_theme",
    "default_color_mode",
    "theme_rotation",
    "home_features",
    "enabled_features",
];

/// 平台保留設定 — 只有 platform:read 能在 GET /admin/settings 看到、platform:update 能改。
/// 商家 instance 的管理員拿 setting:read/update 管日常設定，碰不到這些 key。
const RESERVED_KEYS: &[&str] = &["enabled_features"];

pub fn is_reserved(key: &str) -> bool {
    RESERVED_KEYS.contains(&key)
}

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

/// home_features 驗證：JSON 字串陣列、不重複。只驗形狀不驗 key 名 ——
/// 功能清單由前端 registry（libs/home-features.ts）定義，未知 key 前端會忽略，
/// 新增卡片只需改前端、後端不用同步。
fn validate_home_features(value: &str) -> Result<(), AppError> {
    let items: Vec<String> = serde_json::from_str(value)
        .map_err(|_| unprocessable("home_features 必須是 JSON 字串陣列".into()))?;

    if items.len() > 50 {
        return Err(unprocessable("home_features 最多 50 項".into()));
    }
    let mut seen = std::collections::HashSet::new();
    for item in &items {
        if item.is_empty() || item.len() > 64 {
            return Err(unprocessable("home_features 項目須為 1–64 字元的字串".into()));
        }
        if !seen.insert(item.as_str()) {
            return Err(unprocessable(format!("home_features 有重複項目 {item}")));
        }
    }
    Ok(())
}

/// enabled_features 驗證：`all`，或全部是合法 feature key 的不重複 JSON 字串陣列。
/// 與 home_features 相反這裡驗 key 名 —— feature key 權威在後端 Feature enum，
/// 未知 key = 打錯字或前後端不同步，直接擋下。
fn validate_enabled_features(value: &str) -> Result<(), AppError> {
    if value == "all" {
        return Ok(());
    }
    let items: Vec<String> = serde_json::from_str(value).map_err(|_| {
        unprocessable("enabled_features 必須是 \"all\" 或 JSON 字串陣列".into())
    })?;

    let mut seen = std::collections::HashSet::new();
    for item in &items {
        let Some(feature) = Feature::from_key(item) else {
            let allowed: Vec<&str> = Feature::ALL.iter().map(|f| f.as_str()).collect();
            return Err(unprocessable(format!(
                "enabled_features 有未知功能 {item}，只接受 {}",
                allowed.join(" / ")
            )));
        };
        if !seen.insert(feature) {
            return Err(unprocessable(format!("enabled_features 有重複項目 {item}")));
        }
    }
    // 依賴規則：portfolio 的市價/股名靠 stocks 的排程 job 餵資料
    if seen.contains(&Feature::Portfolio) && !seen.contains(&Feature::Stocks) {
        return Err(unprocessable(
            "enabled_features 啟用 portfolio 時必須同時啟用 stocks".into(),
        ));
    }
    Ok(())
}

/// 設定值驗證 — key 不在表內就不驗證
fn validate(key: &str, value: &str) -> Result<(), AppError> {
    if key == "theme_rotation" {
        return validate_theme_rotation(value);
    }
    if key == "home_features" {
        return validate_home_features(value);
    }
    if key == "enabled_features" {
        return validate_enabled_features(value);
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

/// include_reserved = caller 是否有 platform:read；無則濾掉平台保留 key
pub async fn get_all(
    pool: &Pool<Postgres>,
    include_reserved: bool,
) -> Result<BTreeMap<String, Vec<AppSetting>>, AppError> {
    let rows = repo::get_all(pool).await?;
    let mut grouped: BTreeMap<String, Vec<AppSetting>> = BTreeMap::new();
    for setting in rows {
        if !include_reserved && is_reserved(&setting.key) {
            continue;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_features_accepts_string_array() {
        assert!(validate("home_features", r#"["blog","vocab","about"]"#).is_ok());
        assert!(validate("home_features", "[]").is_ok());
        // 未知 key 名不驗（由前端 registry 過濾）
        assert!(validate("home_features", r#"["not_a_feature"]"#).is_ok());
    }

    #[test]
    fn enabled_features_accepts_all_or_valid_keys() {
        assert!(validate("enabled_features", "all").is_ok());
        assert!(validate("enabled_features", "[]").is_ok());
        assert!(validate("enabled_features", r#"["blog","tools","games"]"#).is_ok());
        assert!(validate("enabled_features", r#"["portfolio","stocks"]"#).is_ok());
    }

    #[test]
    fn enabled_features_rejects_bad_values() {
        assert!(validate("enabled_features", "not json").is_err());
        assert!(validate("enabled_features", r#"{"blog":true}"#).is_err());
        // 未知 key（權威在後端 enum）
        assert!(validate("enabled_features", r#"["not_a_feature"]"#).is_err());
        // 重複
        assert!(validate("enabled_features", r#"["blog","blog"]"#).is_err());
        // portfolio 依賴 stocks
        assert!(validate("enabled_features", r#"["portfolio"]"#).is_err());
    }

    #[test]
    fn home_features_rejects_bad_shape() {
        assert!(validate("home_features", "not json").is_err());
        assert!(validate("home_features", r#"{"blog":true}"#).is_err());
        assert!(validate("home_features", r#"[1,2]"#).is_err());
        assert!(validate("home_features", r#"[""]"#).is_err());
        assert!(validate("home_features", r#"["blog","blog"]"#).is_err());
    }
}

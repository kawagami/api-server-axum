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
    // 前端上傳前壓縮參數（image_webp_quality 只後端讀，不公開）
    "image_client_compress",
    "image_client_quality",
    "image_client_max_edge",
];

/// 平台保留設定 — 只有 platform:read 能在 GET /admin/settings 看到、platform:update 能改。
/// 商家 instance 的管理員拿 setting:read/update 管日常設定，碰不到這些 key。
/// `new_user_default_roles` 決定「新建管理員預設掛哪些角色」，那是平台層的權限決策，
/// 不該只要 setting:update 就能改（否則等於另開一條指派角色的門）。
const RESERVED_KEYS: &[&str] = &[
    "enabled_features",
    "webauthn_rp_id",
    "webauthn_rp_origin",
    "new_user_default_roles",
];

pub fn is_reserved(key: &str) -> bool {
    RESERVED_KEYS.contains(&key)
}

/// 只寫不讀的設定：GET /admin/settings 會把 value 遮掉，PATCH 仍可寫入。
///
/// smtp_password 是 Gmail App Password。原本任何持 setting:read 的人都會在回應 JSON 裡
/// 拿到明文——本站目前只有 super_admin 有這個權限（內建 admin 角色沒有），所以不是現況
/// 外洩；但 instance-per-merchant 的設計正是要讓商家管理員拿 setting:* 管日常設定，
/// 屆時就會變成真的洩漏。先遮起來。
const SECRET_KEYS: &[&str] = &["smtp_password"];

pub fn is_secret(key: &str) -> bool {
    SECRET_KEYS.contains(&key)
}

/// 遮蔽後的顯示值。用固定字串而非空字串，前端才分得出「有設定但不給看」與「沒設定」。
const SECRET_MASK: &str = "********";

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

/// webauthn 設定只驗單值形狀，**不驗兩 key 配對**——用另一半現值驗會讓「整組換新網域」
/// 先存哪個都 422（死鎖，永遠遷不出預設值）。配對規則（rp_id 是 origin 的有效網域）
/// 由前端 /admin/platform 存檔前檢查 + Settings::reload 時建構失敗記 error log、instance 設 None。
fn validate_webauthn_rp_id(value: &str) -> Result<(), AppError> {
    if value.is_empty() || value.contains('/') || value.contains(':') || value.contains(' ') {
        return Err(unprocessable(
            "webauthn_rp_id 必須是裸網域（如 kawa.homes，不含 scheme / port / 路徑）".into(),
        ));
    }
    Ok(())
}

fn validate_webauthn_rp_origin(value: &str) -> Result<(), AppError> {
    let ok = webauthn_rs::prelude::Url::parse(value)
        .map(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
        .unwrap_or(false);
    if !ok {
        return Err(unprocessable(
            "webauthn_rp_origin 必須是合法 http(s) URL（如 https://kawa.homes）".into(),
        ));
    }
    Ok(())
}

/// 設定值驗證 — key 不在表內就不驗證
/// 整數且落在 [min, max]（含）內，否則 422。
fn validate_int_range(key: &str, value: &str, min: u32, max: u32) -> Result<(), AppError> {
    match value.parse::<u32>() {
        Ok(n) if n >= min && n <= max => Ok(()),
        _ => Err(unprocessable(format!("{key} 必須是 {min}–{max} 的整數"))),
    }
}

/// cors_allowed_origins 驗證：逗號分隔，每項必須是 `http(s)://host[:port]`。
///
/// 為什麼一定要擋 `*`：`routes.rs` 啟動時把這個值餵給 tower-http 的
/// `AllowOrigin::list`，而那個建構子**遇到 `*` 會 panic**
/// （tower-http-0.6 的 cors/allow_origin.rs）。CORS 只在啟動時讀，所以改成 `*`
/// 當下不會報錯，但**下一次部署或重啟就會 panic**，配上 compose 的
/// `restart: unless-stopped` 變成無限重啟迴圈，只能直接改 DB 才救得回來。
///
/// 順便擋掉靜默失敗：原本 `filter_map(parse::<HeaderValue>)` 會把打錯的項目
/// 無聲丟掉，最壞變成空清單（所有跨源被拒）而沒有任何告警。
fn validate_cors_allowed_origins(value: &str) -> Result<(), AppError> {
    let items: Vec<&str> = value.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if items.is_empty() {
        return Err(unprocessable("cors_allowed_origins 不可為空".into()));
    }
    for item in items {
        if item.contains('*') {
            return Err(unprocessable(
                "cors_allowed_origins 不接受 *（會讓後端啟動時 panic）；請逐一列出來源".into(),
            ));
        }
        let rest = item
            .strip_prefix("https://")
            .or_else(|| item.strip_prefix("http://"))
            .ok_or_else(|| {
                unprocessable(format!("cors_allowed_origins 的 {item} 必須以 http:// 或 https:// 開頭"))
            })?;
        // origin 只有 scheme://host[:port]，不含路徑／query／fragment
        if rest.is_empty() || rest.contains('/') || rest.contains('?') || rest.contains('#') {
            return Err(unprocessable(format!(
                "cors_allowed_origins 的 {item} 必須是 scheme://host[:port]，不含路徑"
            )));
        }
        // HeaderValue 是最終消費者，先在這裡確認轉得過去
        if item.parse::<axum::http::HeaderValue>().is_err() {
            return Err(unprocessable(format!("cors_allowed_origins 的 {item} 含非法字元")));
        }
    }
    Ok(())
}

fn validate(key: &str, value: &str) -> Result<(), AppError> {
    if key == "theme_rotation" {
        return validate_theme_rotation(value);
    }
    if key == "image_webp_quality" || key == "image_client_quality" {
        return validate_int_range(key, value, 1, 100);
    }
    if key == "image_client_max_edge" {
        // 上限對齊 libwebp 單邊上限 16383
        return validate_int_range(key, value, 64, 16383);
    }
    if key == "image_client_compress" {
        return match value {
            "true" | "false" => Ok(()),
            _ => Err(unprocessable(format!("{key} 只接受 true / false"))),
        };
    }
    if key == "home_features" {
        return validate_home_features(value);
    }
    if key == "enabled_features" {
        return validate_enabled_features(value);
    }
    if key == "webauthn_rp_id" {
        return validate_webauthn_rp_id(value);
    }
    if key == "webauthn_rp_origin" {
        return validate_webauthn_rp_origin(value);
    }
    if key == "cors_allowed_origins" {
        return validate_cors_allowed_origins(value);
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
    for mut setting in rows {
        if !include_reserved && is_reserved(&setting.key) {
            continue;
        }
        // 秘密值一律不出站；有值才遮，沒值就讓它保持空字串
        if is_secret(&setting.key) && !setting.value.is_empty() {
            setting.value = SECRET_MASK.to_string();
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
    fn webauthn_settings_validate_shape_only() {
        // rp_id：裸網域
        assert!(validate("webauthn_rp_id", "kawa.homes").is_ok());
        assert!(validate("webauthn_rp_id", "localhost").is_ok());
        assert!(validate("webauthn_rp_id", "").is_err());
        assert!(validate("webauthn_rp_id", "https://kawa.homes").is_err());
        assert!(validate("webauthn_rp_id", "kawa.homes/admin").is_err());

        // origin：合法 http(s) URL
        assert!(validate("webauthn_rp_origin", "https://kawa.homes").is_ok());
        assert!(validate("webauthn_rp_origin", "http://localhost:3000").is_ok());
        assert!(validate("webauthn_rp_origin", "not-a-url").is_err());
        assert!(validate("webauthn_rp_origin", "ftp://kawa.homes").is_err());
        assert!(validate("webauthn_rp_origin", "").is_err());

        // 不驗配對——整組換新網域時單 key PATCH 不會被另一半現值卡死
        assert!(validate("webauthn_rp_id", "totally-unrelated.example").is_ok());
    }

    #[test]
    fn image_compression_settings_validate() {
        // 品質：1–100 整數
        for key in ["image_webp_quality", "image_client_quality"] {
            assert!(validate(key, "80").is_ok());
            assert!(validate(key, "1").is_ok());
            assert!(validate(key, "100").is_ok());
            assert!(validate(key, "0").is_err());
            assert!(validate(key, "101").is_err());
            assert!(validate(key, "80.5").is_err());
            assert!(validate(key, "high").is_err());
        }
        // 長邊：64–16383 整數
        assert!(validate("image_client_max_edge", "2560").is_ok());
        assert!(validate("image_client_max_edge", "64").is_ok());
        assert!(validate("image_client_max_edge", "16383").is_ok());
        assert!(validate("image_client_max_edge", "63").is_err());
        assert!(validate("image_client_max_edge", "16384").is_err());
        // 開關：true / false
        assert!(validate("image_client_compress", "true").is_ok());
        assert!(validate("image_client_compress", "false").is_ok());
        assert!(validate("image_client_compress", "1").is_err());
        assert!(validate("image_client_compress", "yes").is_err());
    }

    #[test]
    fn home_features_rejects_bad_shape() {
        assert!(validate("home_features", "not json").is_err());
        assert!(validate("home_features", r#"{"blog":true}"#).is_err());
        assert!(validate("home_features", r#"[1,2]"#).is_err());
        assert!(validate("home_features", r#"[""]"#).is_err());
        assert!(validate("home_features", r#"["blog","blog"]"#).is_err());
    }

    /// smtp_password 是 Gmail App Password，不可經 GET /admin/settings 出站
    #[test]
    fn secret_keys_are_masked() {
        assert!(is_secret("smtp_password"));
        assert!(!is_secret("smtp_username"));
        assert!(!is_secret("site_theme"));
    }

    /// new_user_default_roles 決定新管理員的預設角色，屬平台層設定
    #[test]
    fn reserved_keys_cover_role_defaults() {
        assert!(is_reserved("new_user_default_roles"));
        assert!(is_reserved("enabled_features"));
        assert!(!is_reserved("site_theme"));
    }

    #[test]
    fn cors_accepts_valid_origin_lists() {
        assert!(validate("cors_allowed_origins", "https://kawa.homes").is_ok());
        assert!(validate("cors_allowed_origins", "https://kawa.homes,http://localhost:3000").is_ok());
        assert!(validate("cors_allowed_origins", " https://a.example , https://b.example ").is_ok());
    }

    /// 這條是重點：`*` 會讓 AllowOrigin::list 在下次啟動時 panic → 無限重啟迴圈
    #[test]
    fn cors_rejects_wildcard() {
        assert!(validate("cors_allowed_origins", "*").is_err());
        assert!(validate("cors_allowed_origins", "https://kawa.homes,*").is_err());
        assert!(validate("cors_allowed_origins", "https://*.kawa.homes").is_err());
    }

    #[test]
    fn cors_rejects_malformed_entries() {
        assert!(validate("cors_allowed_origins", "").is_err());
        assert!(validate("cors_allowed_origins", "kawa.homes").is_err());          // 缺 scheme
        assert!(validate("cors_allowed_origins", "https://kawa.homes/path").is_err()); // 含路徑
        assert!(validate("cors_allowed_origins", "https://").is_err());
    }
}

use crate::{
    errors::{AppError, RequestError},
    repositories::app_settings as repo,
    state::Settings,
    structs::{app_settings::AppSetting, features::Feature},
};
use sqlx::{Pool, Postgres};
use std::collections::BTreeMap;

/// 公開設定在 `GET /settings/public` 回應裡的型別。
///
/// `app_settings` 的 value 欄是 text，那是**儲存層**的限制，不該滲進 API 契約：
/// 原本整包回 `{key: string}`，於是 `theme_rotation` / `home_features` /
/// `enabled_features` 是「JSON 字串包在 JSON 裡」，每個消費端都得自己再 parse 一次
/// 並各自處理壞值。這裡在出站前轉成該有的型別，前端直接吃。
#[derive(Clone, Copy)]
enum PublicKind {
    /// 原樣輸出字串
    Text,
    /// `"true"` / `"false"` → JSON bool
    Bool,
    /// 整數字串 → JSON number
    Int,
    /// JSON 字面值 → 解析後的物件/陣列；**parse 失敗保留原字串**
    /// （`enabled_features` 的合法值 `all` 就不是 JSON，靠這條走 Text 路徑）
    Json,
}

/// 可由無認證端點讀取的設定白名單 — 新增公開設定時在此加一行（key + 出站型別）
const PUBLIC_KEYS: &[(&str, PublicKind)] = &[
    ("site_theme", PublicKind::Text),
    ("default_color_mode", PublicKind::Text),
    ("theme_rotation", PublicKind::Json),
    ("home_features", PublicKind::Json),
    ("enabled_features", PublicKind::Json),
    // 前端上傳前壓縮參數（image_webp_quality 只後端讀，不公開）
    ("image_client_compress", PublicKind::Bool),
    ("image_client_quality", PublicKind::Int),
    ("image_client_max_edge", PublicKind::Int),
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

/// 這兩支只驗**單值形狀**；兩 key 的配對規則在 `validate_webauthn_pair`。
/// 拆開是因為逐 key PATCH 的中間狀態必然違反配對，整組換網域會死鎖 ——
/// 解法是批次端點 `PATCH /admin/settings` 一次寫兩個 key，配對就驗最終狀態。
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

/// 跨欄位不變式：rp_id 必須是 rp_origin hostname 本身或其上層網域，
/// 否則 WebAuthn ceremony 一律失敗（等於全站 passkey 登入壞掉）。
///
/// 這條規則原本只寫在前端表單（`platform/webauthn-settings.tsx`），後端只驗單值形狀 ——
/// 直接打 API 就能存進互斥的一組值。之所以當初沒收在後端，是因為「用另一半的現值驗配對」
/// 會讓整組換新網域死鎖；解法是 `PATCH /admin/settings`（批次）能一次寫兩個 key，
/// 於是這裡可以無條件檢查最終狀態。
///
/// 任一邊為空 = 尚未設定完成（全新安裝），此時不檢查配對，讓管理員能逐一填入。
fn validate_webauthn_pair(rp_id: &str, rp_origin: &str) -> Result<(), AppError> {
    if rp_id.is_empty() || rp_origin.is_empty() {
        return Ok(());
    }
    let host = webauthn_rs::prelude::Url::parse(rp_origin)
        .ok()
        .and_then(|u| u.host_str().map(String::from));
    let Some(host) = host else {
        // 形狀本身的錯誤由 validate_webauthn_rp_origin 負責報，這裡不重複
        return Ok(());
    };
    if host == rp_id || host.ends_with(&format!(".{rp_id}")) {
        return Ok(());
    }
    Err(unprocessable(format!(
        "webauthn_rp_id（{rp_id}）必須是 webauthn_rp_origin 網域（{host}）本身或其上層網域"
    )))
}

/// 套用這批更新後的最終狀態是否自洽。目前只有 webauthn 這一組跨欄位規則。
fn validate_cross(settings: &Settings, updates: &BTreeMap<String, String>) -> Result<(), AppError> {
    let effective = |key: &str| -> String {
        updates
            .get(key)
            .cloned()
            .or_else(|| settings.get(key))
            .unwrap_or_default()
    };
    if updates.contains_key("webauthn_rp_id") || updates.contains_key("webauthn_rp_origin") {
        validate_webauthn_pair(&effective("webauthn_rp_id"), &effective("webauthn_rp_origin"))?;
    }
    Ok(())
}

/// 把 DB 存的字串轉成出站型別；轉不動一律退回字串（壞值不該讓整個端點壞掉，
/// 前端各 resolver 本來就有 fallback）
fn public_value(kind: PublicKind, raw: String) -> serde_json::Value {
    match kind {
        PublicKind::Text => serde_json::Value::String(raw),
        PublicKind::Bool => match raw.as_str() {
            "true" => serde_json::Value::Bool(true),
            "false" => serde_json::Value::Bool(false),
            _ => serde_json::Value::String(raw),
        },
        PublicKind::Int => match raw.parse::<i64>() {
            Ok(n) => serde_json::Value::Number(n.into()),
            Err(_) => serde_json::Value::String(raw),
        },
        PublicKind::Json => {
            serde_json::from_str(&raw).unwrap_or(serde_json::Value::String(raw))
        }
    }
}

/// 公開設定 — 直接讀記憶體中的 settings map（PATCH 時已自動 reload），不打 DB
pub fn get_public(settings: &Settings) -> BTreeMap<String, serde_json::Value> {
    PUBLIC_KEYS
        .iter()
        .filter_map(|(key, kind)| {
            settings
                .get(key)
                .map(|raw| (key.to_string(), public_value(*kind, raw)))
        })
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
    let mut one = BTreeMap::new();
    one.insert(key.to_string(), value.to_string());
    validate_cross(settings, &one)?;
    let setting = repo::update(pool, key, value).await?;
    settings.reload(pool).await;
    Ok(setting)
}

/// 批次更新：全部驗證通過（含跨欄位）才在同一 transaction 寫入，最後 reload 一次。
///
/// 存在的理由是「互相約束的設定組」—— 逐 key PATCH 時，中間狀態必然違反不變式，
/// 沒有這支就只能把檢查搬到前端（那等於沒有檢查）。
pub async fn update_many(
    pool: &Pool<Postgres>,
    settings: &Settings,
    updates: &BTreeMap<String, String>,
) -> Result<Vec<AppSetting>, AppError> {
    if updates.is_empty() {
        return Err(unprocessable("沒有要更新的設定".into()));
    }
    for (key, value) in updates {
        validate(key, value)?;
    }
    validate_cross(settings, updates)?;

    let pairs: Vec<(String, String)> =
        updates.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let written = repo::update_many(pool, &pairs).await?;
    settings.reload(pool).await;
    Ok(written)
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

        // 單值 validate 只看形狀，配對交給 validate_webauthn_pair（見下）
        assert!(validate("webauthn_rp_id", "totally-unrelated.example").is_ok());
    }

    /// `GET /settings/public` 的出站型別：DB 存字串，但契約上該是布林/數字/物件。
    /// 壞值一律退回字串 —— 前端 resolver 本來就有 fallback，不該讓整個端點噴掉
    #[test]
    fn public_values_are_typed_not_stringly() {
        use serde_json::json;

        assert_eq!(public_value(PublicKind::Text, "forest".into()), json!("forest"));
        assert_eq!(public_value(PublicKind::Bool, "true".into()), json!(true));
        assert_eq!(public_value(PublicKind::Bool, "false".into()), json!(false));
        assert_eq!(public_value(PublicKind::Int, "80".into()), json!(80));
        assert_eq!(
            public_value(PublicKind::Json, r#"["blog","vocab"]"#.into()),
            json!(["blog", "vocab"])
        );
        assert_eq!(
            public_value(PublicKind::Json, r#"{"0":"forest"}"#.into()),
            json!({"0": "forest"})
        );
    }

    #[test]
    fn unparseable_public_values_fall_back_to_the_raw_string() {
        use serde_json::json;

        // enabled_features 的合法值 "all" 不是 JSON —— 必須原樣傳出去
        assert_eq!(public_value(PublicKind::Json, "all".into()), json!("all"));
        assert_eq!(public_value(PublicKind::Json, "壞掉的值".into()), json!("壞掉的值"));
        assert_eq!(public_value(PublicKind::Bool, "yes".into()), json!("yes"));
        assert_eq!(public_value(PublicKind::Int, "abc".into()), json!("abc"));
    }

    /// 配對規則：rp_id 必須是 origin hostname 本身或其上層網域。
    /// 這條原本只在前端表單擋，直接打 API 就能存進互斥的一組值 → passkey 全站失效
    #[test]
    fn webauthn_pair_must_be_consistent() {
        assert!(validate_webauthn_pair("kawa.homes", "https://kawa.homes").is_ok());
        // origin 是子網域、rp_id 是上層 → 合法
        assert!(validate_webauthn_pair("kawa.homes", "https://www.kawa.homes").is_ok());
        assert!(validate_webauthn_pair("localhost", "http://localhost:3000").is_ok());

        // 完全不相干
        assert!(validate_webauthn_pair("kawa.homes", "https://evil.example").is_err());
        // 方向相反：rp_id 比 origin 更深一層
        assert!(validate_webauthn_pair("www.kawa.homes", "https://kawa.homes").is_err());
        // 後綴字串相同但不是網域邊界（kawa.homes vs notkawa.homes）
        assert!(validate_webauthn_pair("kawa.homes", "https://notkawa.homes").is_err());

        // 任一邊尚未設定 → 不檢查，讓全新安裝能逐一填入
        assert!(validate_webauthn_pair("", "https://kawa.homes").is_ok());
        assert!(validate_webauthn_pair("kawa.homes", "").is_ok());
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

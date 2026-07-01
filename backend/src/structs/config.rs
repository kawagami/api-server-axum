pub struct OAuthProviderConfig {
    pub client_secret: String,
}

pub struct AppConfig {
    pub jwt_secret: String,
    /// 是否信任 CF-Connecting-IP header 取得 client IP（僅在確定流量只經 Cloudflare 時開）。
    /// 預設 false：直接用 socket IP，避免 header 偽造繞過 rate limit。
    pub trust_cf_header: bool,
    pub oauth_google: OAuthProviderConfig,
    pub oauth_github: OAuthProviderConfig,
    pub oauth_line: OAuthProviderConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").expect("找不到 JWT_SECRET"),
            trust_cf_header: std::env::var("TRUST_CF_HEADER")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            oauth_google: OAuthProviderConfig {
                client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            },
            oauth_github: OAuthProviderConfig {
                client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
            },
            oauth_line: OAuthProviderConfig {
                client_secret: std::env::var("LINE_CLIENT_SECRET").unwrap_or_default(),
            },
        }
    }
}

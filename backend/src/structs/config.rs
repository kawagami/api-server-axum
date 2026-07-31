pub struct OAuthProviderConfig {
    pub client_secret: String,
}

/// JWT 密鑰的最低長度（bytes）。HS256 的安全性完全取決於這把 key 的熵，
/// 而同一把 secret 簽了 4 種 token（admin access / member access / member refresh /
/// torrent 下載）並與 Next.js 共用。
///
/// 弱密鑰最直接的後果是**偽造 member token**：`authorize_member` 不查 Redis session
/// 也不查 DB，偽造出來的 token 立即生效，可讀寫任一會員的記帳／發票／持股／樂透。
/// 取得一份合法簽名樣本毫無門檻（任何人 OAuth 登入一次就有）。
const MIN_JWT_SECRET_LEN: usize = 32;

/// 啟動期 fail-fast：密鑰太短就不要起來，別讓它悄悄跑在生產上。
fn require_jwt_secret() -> String {
    let secret = std::env::var("JWT_SECRET").expect("找不到 JWT_SECRET");
    assert!(
        secret.len() >= MIN_JWT_SECRET_LEN,
        "JWT_SECRET 長度為 {} bytes，至少需要 {}。請重新產生：openssl rand -base64 48",
        secret.len(),
        MIN_JWT_SECRET_LEN
    );
    secret
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
            jwt_secret: require_jwt_secret(),
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

#[derive(Clone)]
pub struct OAuthProviderConfig {
    pub client_secret: String,
}

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub oauth_google: OAuthProviderConfig,
    pub oauth_github: OAuthProviderConfig,
    pub oauth_line: OAuthProviderConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").expect("找不到 JWT_SECRET"),
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

use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{members, redis},
    state::Settings,
    structs::auth::{Claims, RefreshClaims},
    structs::config::AppConfig,
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use reqwest::Client;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

struct OAuthConfig {
    client_id: String,
    client_secret: String,
    redirect_url: String,
    auth_url: &'static str,
    token_url: &'static str,
}

pub enum OAuthProvider {
    Google,
    GitHub,
    Line,
}

impl OAuthProvider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "google" => Some(Self::Google),
            "github" => Some(Self::GitHub),
            "line" => Some(Self::Line),
            _ => None,
        }
    }

    fn config_from(&self, config: &AppConfig, settings: &Settings) -> Result<OAuthConfig, AppError> {
        let (client_secret, id_key, redirect_key, auth_url, token_url) = match self {
            Self::Google => (
                &config.oauth_google.client_secret,
                "google_client_id",
                "google_redirect_url",
                "https://accounts.google.com/o/oauth2/v2/auth",
                "https://oauth2.googleapis.com/token",
            ),
            Self::GitHub => (
                &config.oauth_github.client_secret,
                "github_client_id",
                "github_redirect_url",
                "https://github.com/login/oauth/authorize",
                "https://github.com/login/oauth/access_token",
            ),
            Self::Line => (
                &config.oauth_line.client_secret,
                "line_client_id",
                "line_redirect_url",
                "https://access.line.me/oauth2/v2.1/authorize",
                "https://api.line.me/oauth2/v2.1/token",
            ),
        };

        let client_id = settings
            .get(id_key)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                AppError::SystemError(SystemError::Internal(format!("{} not configured", id_key)))
            })?;

        let redirect_url = settings
            .get(redirect_key)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                AppError::SystemError(SystemError::Internal(format!(
                    "{} not configured",
                    redirect_key
                )))
            })?;

        Ok(OAuthConfig {
            client_id,
            client_secret: client_secret.clone(),
            redirect_url,
            auth_url,
            token_url,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::GitHub => "github",
            Self::Line => "line",
        }
    }
}

pub fn get_oauth_url(state_value: &str, provider: &OAuthProvider, config: &AppConfig, settings: &Settings) -> Result<String, AppError> {
    let cfg = provider.config_from(config, settings)?;

    let scope = match provider {
        OAuthProvider::Google => "openid email profile",
        OAuthProvider::GitHub => "read:user user:email",
        OAuthProvider::Line => "profile",
    };

    let encode = |s: &str| form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>();

    Ok(format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        cfg.auth_url,
        encode(&cfg.client_id),
        encode(&cfg.redirect_url),
        encode(scope),
        encode(state_value),
    ))
}

pub async fn generate_oauth_url(
    redis_pool: &RedisPool<RedisConnectionManager>,
    config: &AppConfig,
    settings: &Settings,
    provider: &OAuthProvider,
) -> Result<String, AppError> {
    let state_value = Uuid::new_v4().to_string();
    redis::set_oauth_state(redis_pool, &state_value).await?;
    get_oauth_url(&state_value, provider, config, settings)
}

pub async fn exchange_code(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    config: &AppConfig,
    settings: &Settings,
    client: &Client,
    provider: &OAuthProvider,
    code: &str,
    state_param: &str,
) -> Result<(String, String), AppError> {
    let valid = redis::consume_oauth_state(redis_pool, state_param).await?;
    if !valid {
        return Err(AppError::AuthError(AuthError::InvalidToken));
    }

    let cfg = provider.config_from(config, settings)?;
    let token_res = exchange_code_for_token(client, provider, &cfg, code).await?;
    let user_info = fetch_user_info(client, provider, &token_res.access_token).await?;

    let member_id = members::find_or_create_by_oauth(
        pool,
        provider.name(),
        &user_info.provider_id,
        &user_info.name,
        user_info.email.as_deref(),
        user_info.avatar_url.as_deref(),
    )
    .await?;

    issue_tokens(redis_pool, config, member_id).await
}

pub async fn refresh_member_token(
    redis_pool: &RedisPool<RedisConnectionManager>,
    config: &AppConfig,
    refresh_token_jwt: &str,
) -> Result<(String, String), AppError> {
    let refresh_claims = decode_refresh_jwt(refresh_token_jwt, &config.jwt_secret)?;
    let member_id: i64 = refresh_claims
        .sub
        .parse()
        .map_err(|_| AppError::AuthError(AuthError::InvalidToken))?;

    let stored_jti = redis::get_member_refresh_token(redis_pool, member_id)
        .await?
        .ok_or(AppError::AuthError(AuthError::Unauthorized))?;

    if stored_jti != refresh_claims.jti {
        return Err(AppError::AuthError(AuthError::Unauthorized));
    }

    issue_tokens(redis_pool, config, member_id).await
}

async fn issue_tokens(
    redis_pool: &RedisPool<RedisConnectionManager>,
    config: &AppConfig,
    member_id: i64,
) -> Result<(String, String), AppError> {
    let jti = Uuid::new_v4().to_string();
    redis::set_member_refresh_token(redis_pool, member_id, &jti).await?;

    let access_token = encode_member_jwt(member_id, &config.jwt_secret)?;
    let refresh_token = encode_refresh_jwt(member_id, &jti, &config.jwt_secret)?;

    Ok((access_token, refresh_token))
}

fn encode_member_jwt(member_id: i64, jwt_secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claim = Claims {
        iat: now.timestamp() as usize,
        exp: (now + Duration::hours(1)).timestamp() as usize,
        sub: member_id.to_string(),
        role: "member".to_string(),
    };

    encode(&Header::default(), &claim, &EncodingKey::from_secret(jwt_secret.as_ref()))
        .map_err(|_| AppError::AuthError(AuthError::InvalidToken))
}

fn encode_refresh_jwt(member_id: i64, jti: &str, jwt_secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claim = RefreshClaims {
        iat: now.timestamp() as usize,
        exp: (now + Duration::days(30)).timestamp() as usize,
        sub: member_id.to_string(),
        jti: jti.to_string(),
    };

    encode(&Header::default(), &claim, &EncodingKey::from_secret(jwt_secret.as_ref()))
        .map_err(|_| AppError::AuthError(AuthError::InvalidToken))
}

fn decode_refresh_jwt(jwt: &str, jwt_secret: &str) -> Result<RefreshClaims, AppError> {
    decode::<RefreshClaims>(
        jwt,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::AuthError(AuthError::TokenExpired),
        _ => AppError::AuthError(AuthError::InvalidToken),
    })
}

struct ProviderTokenResponse {
    access_token: String,
}

struct OAuthUserInfo {
    provider_id: String,
    name: String,
    email: Option<String>,
    avatar_url: Option<String>,
}

async fn exchange_code_for_token(
    client: &reqwest::Client,
    _provider: &OAuthProvider,
    cfg: &OAuthConfig,
    code: &str,
) -> Result<ProviderTokenResponse, AppError> {
    let params = [
        ("client_id", cfg.client_id.as_str()),
        ("client_secret", cfg.client_secret.as_str()),
        ("code", code),
        ("redirect_uri", cfg.redirect_url.as_str()),
        ("grant_type", "authorization_code"),
    ];

    let res = client
        .post(cfg.token_url)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::SystemError(SystemError::Internal(
            "OAuth token exchange failed".to_string(),
        )));
    }

    let body: serde_json::Value = res.json().await?;
    let access_token = body["access_token"]
        .as_str()
        .ok_or_else(|| AppError::SystemError(SystemError::Internal("missing access_token".to_string())))?
        .to_string();

    Ok(ProviderTokenResponse { access_token })
}

async fn fetch_user_info(
    client: &reqwest::Client,
    provider: &OAuthProvider,
    access_token: &str,
) -> Result<OAuthUserInfo, AppError> {
    match provider {
        OAuthProvider::Google => fetch_google_user(client, access_token).await,
        OAuthProvider::GitHub => fetch_github_user(client, access_token).await,
        OAuthProvider::Line => fetch_line_user(client, access_token).await,
    }
}

async fn fetch_google_user(client: &reqwest::Client, access_token: &str) -> Result<OAuthUserInfo, AppError> {
    #[derive(Deserialize)]
    struct GoogleUser {
        sub: String,
        name: String,
        email: Option<String>,
        picture: Option<String>,
    }

    let user: GoogleUser = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .json()
        .await?;

    Ok(OAuthUserInfo {
        provider_id: user.sub,
        name: user.name,
        email: user.email,
        avatar_url: user.picture,
    })
}

async fn fetch_github_user(client: &reqwest::Client, access_token: &str) -> Result<OAuthUserInfo, AppError> {
    #[derive(Deserialize)]
    struct GitHubUser {
        id: i64,
        name: Option<String>,
        email: Option<String>,
        avatar_url: Option<String>,
        login: String,
    }

    let user: GitHubUser = client
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("User-Agent", "template-axum")
        .send()
        .await?
        .json()
        .await?;

    let email = if user.email.is_some() {
        user.email
    } else {
        fetch_github_primary_email(client, access_token).await.unwrap_or(None)
    };

    Ok(OAuthUserInfo {
        provider_id: user.id.to_string(),
        name: user.name.unwrap_or(user.login),
        email,
        avatar_url: user.avatar_url,
    })
}

async fn fetch_github_primary_email(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<Option<String>, AppError> {
    #[derive(Deserialize)]
    struct GitHubEmail {
        email: String,
        primary: bool,
        verified: bool,
    }

    let emails: Vec<GitHubEmail> = client
        .get("https://api.github.com/user/emails")
        .bearer_auth(access_token)
        .header("User-Agent", "template-axum")
        .send()
        .await?
        .json()
        .await?;

    Ok(emails.into_iter().find(|e| e.primary && e.verified).map(|e| e.email))
}

async fn fetch_line_user(client: &reqwest::Client, access_token: &str) -> Result<OAuthUserInfo, AppError> {
    #[derive(Deserialize)]
    struct LineProfile {
        #[serde(rename = "userId")]
        user_id: String,
        #[serde(rename = "displayName")]
        display_name: String,
        #[serde(rename = "pictureUrl")]
        picture_url: Option<String>,
    }

    let profile: LineProfile = client
        .get("https://api.line.me/v2/profile")
        .bearer_auth(access_token)
        .send()
        .await?
        .json()
        .await?;

    Ok(OAuthUserInfo {
        provider_id: profile.user_id,
        name: profile.display_name,
        email: None,
        avatar_url: profile.picture_url,
    })
}

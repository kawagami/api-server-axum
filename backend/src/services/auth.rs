use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, roles as roles_repo, users},
    structs::auth::Claims,
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::{Pool, Postgres};
use std::sync::LazyLock;

// 帳號不存在時也要跑一次同 cost 的 bcrypt，拉平回應時間防 timing 枚舉。
// lazy 初始化發生在 spawn_blocking 執行緒，不佔 async worker。
static DUMMY_HASH: LazyLock<String> =
    LazyLock::new(|| hash("dummy-password", DEFAULT_COST).expect("bcrypt dummy hash"));

pub async fn sign_in(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    name: &str,
    password: &str,
) -> Result<String, AppError> {
    let (id, password_hash) = match users::get_credentials_by_name(pool, name).await? {
        Some(cred) => cred,
        None => {
            // 帳號不存在也跑一次 bcrypt，拉平回應時間防 timing 枚舉
            dummy_verify_password(password.to_string()).await;
            return Err(AppError::AuthError(AuthError::InvalidCredentials));
        }
    };

    if !verify_password(password.to_string(), password_hash).await? {
        return Err(AppError::AuthError(AuthError::InvalidCredentials));
    }

    complete_admin_login(pool, redis_pool, jwt_secret, id).await
}

/// 身分驗證通過後的共同收尾（密碼與 passkey 登入共用）：
/// Redis 寫 login key + 快取 permissions + 簽發 JWT。
pub async fn complete_admin_login(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    id: i64,
) -> Result<String, AppError> {
    let login_key = format!("user:login:{}", id);
    redis::redis_set(redis_pool, &login_key, &id.to_string()).await?;

    let permissions = roles_repo::get_user_permission_strings_by_id(pool, id).await?;
    redis::set_user_permissions(redis_pool, id, &permissions).await?;

    encode_jwt(id, jwt_secret)
}

pub async fn refresh_admin_token(
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    id: i64,
) -> Result<String, AppError> {
    let login_key = format!("user:login:{}", id);
    redis::redis_set(redis_pool, &login_key, &id.to_string()).await?;
    encode_jwt(id, jwt_secret)
}

/// 改密碼。成功後撤銷該帳號的登入 session（清 Redis `user:login:{id}`）——
/// 改密碼的動機常常是「懷疑帳號被盜」，若不撤銷，被竊的 token 還能再用最多 1 小時。
/// 代價：操作者自己也會被登出，需以新密碼重新登入（前端會導回登入頁）。
pub async fn change_password(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    id: i64,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let current_hash = users::get_password_by_id(pool, id)
        .await?
        .ok_or(AppError::AuthError(AuthError::UserNotFound))?;

    if !verify_password(current_password.to_string(), current_hash).await? {
        return Err(AppError::AuthError(AuthError::InvalidPassword));
    }

    let new_hash = hash_password(new_password.to_string()).await?;
    users::update_password(pool, id, &new_hash).await?;

    // 密碼已經換了，這一步失敗只代表舊 session 多活最多 1 小時（access token 效期），
    // 不該把整個改密碼判定成失敗 —— 但要留下可查的 error log。
    if let Err(e) = crate::repositories::redis::del_user_login(redis_pool, id).await {
        tracing::error!("change_password: 撤銷 user:login:{id} 失敗: {e}");
    }
    Ok(())
}

fn encode_jwt(id: i64, jwt_secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(1)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    // sub 存 user id（字串），與前台 member token 一致；顯示名 name 不進 token
    let claim = Claims { iat, exp, sub: id.to_string(), role: "admin".to_string() };

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::AuthError(AuthError::InvalidToken))
}

async fn dummy_verify_password(password: String) {
    let _ = tokio::task::spawn_blocking(move || verify(password, &DUMMY_HASH)).await;
}

// bcrypt 為 CPU-bound（DEFAULT_COST 約百毫秒），用 spawn_blocking 避免卡住 tokio worker
async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || verify(password, &hash))
        .await
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼驗證 task 失敗".to_string())))?
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼驗證處理失敗".to_string())))
}

pub(crate) async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || hash(password, DEFAULT_COST))
        .await
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼 hash task 失敗".to_string())))?
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼 hash 失敗".to_string())))
}

/// JWT 簽驗的煙霧測試 —— 存在的唯一理由是守住 `Cargo.toml` 的 jsonwebtoken feature。
///
/// v10 起 crypto backend 改成執行期查表（`CryptoProvider`），少挑一個 backend
/// **編譯完全過得去**，clippy 也不會吭聲，直到真的簽 token 才 panic
/// （"Could not automatically determine the process-level CryptoProvider"）。
/// 那條路徑上是登入、middleware 驗 token、torrent 簽名連結 —— 等於全站認證掛掉。
/// 2026-08-02 Dependabot 的升版 PR 就是這個形態：CI 全綠，因為當時沒有任何測試碰 JWT。
#[cfg(test)]
mod jwt_crypto_provider {
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    fn claims() -> Claims {
        Claims {
            sub: "smoke@test".to_string(),
            exp: 9_999_999_999,
        }
    }

    #[test]
    fn hs256_sign_then_verify() {
        let token = encode(
            &Header::default(),
            &claims(),
            &EncodingKey::from_secret(b"secret"),
        )
        .expect("簽發失敗");

        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(b"secret"),
            &Validation::default(),
        )
        .expect("驗證失敗");

        assert_eq!(decoded.claims.sub, "smoke@test");
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let token = encode(
            &Header::default(),
            &claims(),
            &EncodingKey::from_secret(b"secret"),
        )
        .expect("簽發失敗");

        assert!(decode::<Claims>(
            &token,
            &DecodingKey::from_secret(b"another-secret"),
            &Validation::default(),
        )
        .is_err());
    }
}

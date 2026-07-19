use crate::{errors::AppError, structs::webauthn::PasskeyListItem};
use sqlx::{Pool, Postgres};

/// 管理頁列表用：不回公鑰 / credential_id。
pub async fn list_by_user_id(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<Vec<PasskeyListItem>, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, label, created_at, last_used_at
         FROM user_passkeys WHERE user_id = $1 ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// 登入 finish / 註冊 begin（exclude_credentials）用：撈該 user 全部序列化 passkey。
pub async fn passkeys_by_user_id(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    Ok(sqlx::query_scalar(
        "SELECT passkey FROM user_passkeys WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// 回 false = credential_id 已存在（撞 UNIQUE，caller 回 409）。
pub async fn insert(
    pool: &Pool<Postgres>,
    user_id: i64,
    credential_id: &str,
    passkey: &serde_json::Value,
    label: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        "INSERT INTO user_passkeys (user_id, credential_id, passkey, label)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (credential_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(credential_id)
    .bind(passkey)
    .bind(label)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// 只能刪自己的；回 false = 不存在或非本人（caller 回 404）。
pub async fn delete_own(
    pool: &Pool<Postgres>,
    user_id: i64,
    id: i64,
) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM user_passkeys WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// 登入成功後回寫：counter/backup 旗標有變時傳 Some(passkey) 更新整包，否則只更 last_used_at。
pub async fn touch_after_auth(
    pool: &Pool<Postgres>,
    credential_id: &str,
    updated_passkey: Option<&serde_json::Value>,
) -> Result<(), AppError> {
    match updated_passkey {
        Some(passkey) => {
            sqlx::query(
                "UPDATE user_passkeys SET passkey = $1, last_used_at = now()
                 WHERE credential_id = $2",
            )
            .bind(passkey)
            .bind(credential_id)
            .execute(pool)
            .await?;
        }
        None => {
            sqlx::query(
                "UPDATE user_passkeys SET last_used_at = now() WHERE credential_id = $1",
            )
            .bind(credential_id)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

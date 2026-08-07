use crate::{errors::AppError, structs::app_settings::AppSetting};
use sqlx::{Pool, Postgres};

pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<AppSetting>, AppError> {
    Ok(
        sqlx::query_as(
            "SELECT key, value, description, category FROM app_settings ORDER BY category, key",
        )
        .fetch_all(pool)
        .await?,
    )
}

pub async fn update(pool: &Pool<Postgres>, key: &str, value: &str) -> Result<AppSetting, AppError> {
    Ok(
        sqlx::query_as(
            "UPDATE app_settings SET value = $2 WHERE key = $1 RETURNING key, value, description, category",
        )
        .bind(key)
        .bind(value)
        .fetch_one(pool)
        .await?,
    )
}

/// 一個 transaction 內更新多個 key —— 讓「兩個值互相約束」的設定組（webauthn_rp_id /
/// webauthn_rp_origin）能一次換掉，不會中途留下互斥的一半。回傳順序同輸入。
pub async fn update_many(
    pool: &Pool<Postgres>,
    updates: &[(String, String)],
) -> Result<Vec<AppSetting>, AppError> {
    let mut tx = pool.begin().await?;
    let mut out = Vec::with_capacity(updates.len());
    for (key, value) in updates {
        let setting: AppSetting = sqlx::query_as(
            "UPDATE app_settings SET value = $2 WHERE key = $1 RETURNING key, value, description, category",
        )
        .bind(key)
        .bind(value)
        .fetch_one(&mut *tx)
        .await?;
        out.push(setting);
    }
    tx.commit().await?;
    Ok(out)
}

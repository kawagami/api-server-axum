use crate::{
    errors::{unprocessable, AppError, RequestError},
    repositories::vocab as vocab_repo,
    services::vocab_ja,
    structs::vocab::{AdminWordListQuery, AdminWordListResponse, UpdateWordRequest},
};

// ---- 後台題庫管理（/admin/vocab）----

/// 題庫分頁列表（含全會員答錯統計）
pub async fn admin_list_words(
    pool: &sqlx::Pool<sqlx::Postgres>,
    filter: &AdminWordListQuery,
    limit: i64,
    offset: i64,
) -> Result<AdminWordListResponse, AppError> {
    let (data, total) = vocab_repo::admin_list(pool, filter, limit, offset).await?;
    Ok(AdminWordListResponse { data, total })
}

/// 更新單字（釋義／讀音／難度／上下架；**表記與語言不可改**，改表記走 seed migration）。
///
/// 驗證與寫入綁在同一支：日文的「主讀音必須在 accepted_readings 內」是**答題會不會被
/// 誤判**的規則（拼字題答主讀音卻被判錯），不是表單檢查，屬於這一層。
pub async fn admin_update_word(
    pool: &sqlx::Pool<sqlx::Postgres>,
    id: i64,
    req: &mut UpdateWordRequest,
) -> Result<(), AppError> {
    if !(1..=5).contains(&req.difficulty) {
        return Err(unprocessable("難度須在 1–5"));
    }

    let language = vocab_repo::admin_word_language(pool, id)
        .await?
        .ok_or(RequestError::NotFound)?;

    if language == "ja" {
        // 日文必有讀音（DB CHECK 也擋，這裡先給友善錯誤）
        let reading = req.reading.as_deref().map(str::trim).unwrap_or_default();
        if reading.is_empty() {
            return Err(unprocessable("日文單字必須有讀音"));
        }
        let normalized = vocab_ja::normalize_reading(reading);
        let accepted = req.accepted_readings.get_or_insert_with(Vec::new);
        accepted.retain(|r| !r.trim().is_empty());
        if !accepted.iter().any(|r| vocab_ja::normalize_reading(r) == normalized) {
            accepted.insert(0, reading.to_string());
        }
    }

    vocab_repo::admin_update_word(pool, id, req).await
}

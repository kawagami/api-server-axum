use crate::{
    errors::{AppError, RequestError},
    state::AppStateV2,
    structs::stocks::{Conditions, StockChange, StockChangeWithoutId, StockRequest},
};
use sqlx::{QueryBuilder, Row};

pub async fn save_request(state: &AppStateV2, payload: &StockRequest) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO stock_changes (stock_no, start_date, end_date, status, created_at, updated_at)
        VALUES ($1, $2, $3, 'pending', now(), now())",
    )
    .bind(&payload.stock_no)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .execute(state.get_pool())
    .await?;

    Ok(())
}

pub async fn get_all_stock_changes(
    state: &AppStateV2,
    conditions: Conditions,
) -> Result<Vec<StockChange>, AppError> {
    let mut query = QueryBuilder::new(
        "SELECT * FROM stock_changes s WHERE 1=1",
    );

    if let Some(status) = &conditions.status {
        query.push(" AND s.status = ");
        query.push_bind(status);
    }

    query.push(" ORDER BY s.start_date DESC");

    Ok(query.build_query_as().fetch_all(state.get_pool()).await?)
}

pub async fn get_stock_change_info(
    state: &AppStateV2,
    stock_form: &StockRequest,
) -> Result<StockChangeWithoutId, AppError> {
    let url = format!("{}{}", state.get_fastapi_upload_host(), "/stock-change");

    let response = state
        .get_http_client()
        .post(url)
        .json(&stock_form)
        .send()
        .await
        .map_err(|err| AppError::ConnectionError(err.into()))?;

    if !response.status().is_success() {
        return Err(RequestError::InvalidContent(format!(
            "Server returned status code: {}",
            response.status()
        ))
        .into());
    }

    Ok(response.json::<StockChangeWithoutId>().await?)
}

pub async fn get_one_pending_stock_change(
    state: &AppStateV2,
) -> Result<Option<StockRequest>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT stock_no, start_date, end_date
        FROM stock_changes
        WHERE status = 'pending'
            AND TO_DATE(
                (CAST((CAST(end_date AS TEXT)::INT + 19110000) AS TEXT)),
                'YYYYMMDD'
            ) <= CURRENT_DATE
        LIMIT 1
        "#,
    )
    .fetch_optional(state.get_pool())
    .await?;

    if let Some(row) = row {
        Ok(Some(StockRequest {
            stock_no: row.get("stock_no"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn upsert_stock_change(
    state: &AppStateV2,
    info: &StockChangeWithoutId,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO stock_changes (
            stock_no, stock_name, start_date, start_price,
            end_date, end_price, change, status, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'completed', now(), now())
        ON CONFLICT (stock_no, start_date, end_date)
        DO UPDATE SET
            status = 'completed',
            stock_name = EXCLUDED.stock_name,
            start_price = EXCLUDED.start_price,
            end_price = EXCLUDED.end_price,
            change = EXCLUDED.change,
            updated_at = now()
        "#,
    )
    .bind(&info.stock_no)
    .bind(&info.stock_name)
    .bind(&info.start_date)
    .bind(&info.start_price)
    .bind(&info.end_date)
    .bind(&info.end_price)
    .bind(&info.change)
    .execute(state.get_pool())
    .await?;

    Ok(())
}

pub async fn get_existing_stock_change(
    state: &AppStateV2,
    payload: &StockRequest,
) -> Result<Option<StockChangeWithoutId>, AppError> {
    Ok(sqlx::query_as::<_, StockChangeWithoutId>(
        r#"
        SELECT stock_no, start_date, end_date, status, stock_name, start_price, end_price, change
        FROM stock_changes
        WHERE stock_no = $1 AND start_date = $2 AND end_date = $3
        "#,
    )
    .bind(&payload.stock_no)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .fetch_optional(state.get_pool())
    .await?)
}

pub async fn insert_stock_data_batch(
    state: &AppStateV2,
    stocks: &[StockRequest],
) -> Result<usize, AppError> {
    let mut tx = state.get_pool().begin().await?;

    let stock_nos: Vec<&str> = stocks.iter().map(|s| s.stock_no.as_str()).collect();
    let start_dates: Vec<&str> = stocks.iter().map(|s| s.start_date.as_str()).collect();
    let end_dates: Vec<&str> = stocks.iter().map(|s| s.end_date.as_str()).collect();
    let statuses: Vec<&str> = vec!["pending"; stocks.len()];
    let now = chrono::Utc::now().naive_utc();
    let timestamps: Vec<chrono::NaiveDateTime> = vec![now; stocks.len()];

    sqlx::query(
        "INSERT INTO stock_changes (stock_no, start_date, end_date, status, created_at, updated_at)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::timestamptz[], $6::timestamptz[])
        ON CONFLICT (stock_no, start_date, end_date) DO NOTHING",
    )
    .bind(&stock_nos)
    .bind(&start_dates)
    .bind(&end_dates)
    .bind(&statuses)
    .bind(&timestamps)
    .bind(&timestamps)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(stocks.len())
}

pub async fn update_stock_change_failed(
    state: &AppStateV2,
    stock: &StockRequest,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE stock_changes SET updated_at = NOW(), status = 'failed'
        WHERE stock_no = $1 AND start_date = $2 AND end_date = $3",
    )
    .bind(&stock.stock_no)
    .bind(&stock.start_date)
    .bind(&stock.end_date)
    .execute(state.get_pool())
    .await?;

    Ok(())
}

pub async fn reset_failed_stock_changes_to_pending(state: &AppStateV2) -> Result<(), AppError> {
    sqlx::query(
        r#"UPDATE stock_changes SET "status" = 'pending', updated_at = NOW() WHERE "status" = 'failed'"#,
    )
    .execute(state.get_pool())
    .await?;

    Ok(())
}

pub async fn update_one_stock_change_pending(state: &AppStateV2, id: i32) -> Result<(), AppError> {
    sqlx::query(
        r#"UPDATE stock_changes
        SET "status" = 'pending', stock_name = NULL, start_price = NULL,
            end_price = NULL, change = NULL, updated_at = NOW()
        WHERE id = $1"#,
    )
    .bind(id)
    .execute(state.get_pool())
    .await?;

    Ok(())
}

pub async fn check_stock_change_pending_exist(
    state: &AppStateV2,
    payload: &StockRequest,
) -> Result<Option<StockChange>, AppError> {
    Ok(sqlx::query_as(
        "SELECT stock_no, start_date, end_date, stock_name, start_price, end_price, change
        FROM stock_changes
        WHERE stock_no = $1 AND start_date = $2 AND end_date = $3 AND status = 'pending'",
    )
    .bind(&payload.stock_no)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .fetch_optional(state.get_pool())
    .await?)
}

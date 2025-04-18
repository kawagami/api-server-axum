use crate::{
    errors::{AppError, RequestError},
    state::AppStateV2,
    structs::stocks::{Conditions, Stock, StockChange, StockChangeWithoutId, StockRequest},
};
use sqlx::{QueryBuilder, Row};

pub async fn fetch_stock_day_avg_all(state: &AppStateV2) -> Result<Vec<Stock>, AppError> {
    let url = "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_AVG_ALL";

    Ok(state
        .get_http_client()
        .get(url)
        .send()
        .await?
        .json::<Vec<Stock>>()
        .await?)
}

pub async fn save_stock_day_avg_all(state: &AppStateV2, stocks: &[Stock]) -> Result<usize, AppError> {
    let mut tx = state.get_pool().begin().await?;

    let query = "
        INSERT INTO stocks (code, name, closing_price, monthly_average_price)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::float8[], $4::float8[])
        ON CONFLICT (code) DO UPDATE 
        SET name = EXCLUDED.name,
            closing_price = EXCLUDED.closing_price,
            monthly_average_price = EXCLUDED.monthly_average_price;
    ";

    let codes: Vec<&str> = stocks.iter().map(|s| s.code.as_str()).collect();
    let names: Vec<&str> = stocks.iter().map(|s| s.name.as_str()).collect();
    let closing_prices: Vec<f64> = stocks
        .iter()
        .map(|s| s.closing_price.parse().unwrap_or(0.0))
        .collect();
    let monthly_avg_prices: Vec<f64> = stocks
        .iter()
        .map(|s| s.monthly_average_price.parse().unwrap_or(0.0))
        .collect();

    sqlx::query(query)
        .bind(&codes)
        .bind(&names)
        .bind(&closing_prices)
        .bind(&monthly_avg_prices)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(stocks.len())
}

pub async fn save_request(state: &AppStateV2, payload: &StockRequest) -> Result<(), AppError> {
    let pool = state.get_pool();
    let query = "
        INSERT INTO stock_changes (stock_no, start_date, end_date, status, created_at, updated_at)
        VALUES ($1, $2, $3, 'pending', now(), now())
    ";
    sqlx::query(query)
        .bind(&payload.stock_no)
        .bind(&payload.start_date)
        .bind(&payload.end_date)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_all_stock_changes(
    state: &AppStateV2,
    conditions: Conditions,
) -> Result<Vec<StockChange>, AppError> {
    let mut query = QueryBuilder::new(
        r#"
        SELECT
            *
        FROM
            stock_changes s
        WHERE 1=1
    "#,
    );

    // Add status condition if it exists
    if let Some(status) = &conditions.status {
        query.push(" AND s.status = ");
        query.push_bind(status);
    }

    // Add the ordering at the end
    query.push(" ORDER BY s.start_date DESC");

    // Execute the query with the arguments
    let requests: Vec<StockChange> = query.build_query_as().fetch_all(state.get_pool()).await?;

    Ok(requests)
}

/// 使用 fastapi 獲取股票期間差異
pub async fn get_stock_change_info(
    state: &AppStateV2,
    stock_form: &StockRequest,
) -> Result<StockChangeWithoutId, AppError> {
    let client = state.get_http_client();
    let url = format!("{}{}", state.get_fastapi_upload_host(), "/stock-change");

    let response = client
        .post(url)
        .json(&stock_form)
        .send()
        .await
        .map_err(|err| AppError::ConnectionError(err.into()))?;

    // 先檢查狀態碼
    if !response.status().is_success() {
        return Err(AppError::RequestError(RequestError::InvalidContent(
            format!("Server returned status code: {}", response.status()),
        )));
    }

    Ok(response.json::<StockChangeWithoutId>().await?)
}

pub async fn get_one_pending_stock_change(
    state: &AppStateV2,
) -> anyhow::Result<Option<StockRequest>> {
    let row = sqlx::query(
        r#"
        SELECT
            stock_no,
            start_date,
            end_date
        FROM
            stock_changes
        WHERE
            status = 'pending'
        LIMIT
            1
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
            stock_no,
            stock_name,
            start_date,
            start_price,
            end_date,
            end_price,
            change,
            status,
            created_at,
            updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, 'completed', now(), now()
        )
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

/// 查詢是否已存在特定條件的 stock_change 記錄
pub async fn get_existing_stock_change(
    state: &AppStateV2,
    payload: &StockRequest,
) -> Result<Option<StockChangeWithoutId>, AppError> {
    let existing_info = sqlx::query_as::<_, StockChangeWithoutId>(
        r#"
        SELECT
            stock_no,
            start_date,
            end_date,
            status,
            stock_name,
            start_price,
            end_price,
            change
        FROM
            stock_changes
        WHERE
            stock_no = $1
            AND start_date = $2
            AND end_date = $3
        "#,
    )
    .bind(&payload.stock_no)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .fetch_optional(state.get_pool())
    .await?;

    Ok(existing_info)
}

pub async fn insert_stock_data_batch(
    state: &AppStateV2,
    stocks: &[StockRequest],
) -> Result<usize, AppError> {
    let mut tx = state.get_pool().begin().await?;

    let query = "
        INSERT INTO stock_changes (stock_no, start_date, end_date, status, created_at, updated_at)
        SELECT * FROM UNNEST(
            $1::text[], $2::text[], $3::text[], 
            $4::text[], $5::timestamptz[], $6::timestamptz[]
        )
        ON CONFLICT (stock_no, start_date, end_date) DO NOTHING;
    ";

    let stock_nos: Vec<&str> = stocks.iter().map(|s| s.stock_no.as_str()).collect();
    let start_dates: Vec<&str> = stocks.iter().map(|s| s.start_date.as_str()).collect();
    let end_dates: Vec<&str> = stocks.iter().map(|s| s.end_date.as_str()).collect();
    let statuses: Vec<&str> = vec!["pending"; stocks.len()]; // 預設 'pending'
    let timestamps: Vec<&str> = vec!["NOW()"; stocks.len()]; // `NOW()`

    sqlx::query(query)
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

// 將打 fastapi 失敗的資料改成 failed
pub async fn update_stock_change_failed(
    state: &AppStateV2,
    stock: &StockRequest,
) -> Result<(), AppError> {
    let mut tx = state.get_pool().begin().await?;

    // status 欄位改成 failed 的 update sql where
    let query = r#"
            UPDATE stock_changes
            SET
                updated_at = NOW(),
                status = 'failed'
            WHERE
                stock_no = $1
                AND start_date = $2
                AND end_date = $3
        "#;

    sqlx::query(query)
        .bind(&stock.stock_no)
        .bind(&stock.start_date)
        .bind(&stock.end_date)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn reset_failed_stock_changes_to_pending(state: &AppStateV2) -> Result<(), AppError> {
    let mut tx = state.get_pool().begin().await?;

    // status 欄位改成 failed 的 update sql where
    let query = r#"
            UPDATE stock_changes
            SET
                "status" = 'pending',
                updated_at = NOW ()
            WHERE
                "status" = 'failed';
        "#;

    sqlx::query(query).fetch_all(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn update_one_stock_change_pending(state: &AppStateV2, id: i32) -> Result<(), AppError> {
    let mut tx = state.get_pool().begin().await?;

    // status 欄位改成 failed 的 update sql where
    let query = r#"
            UPDATE stock_changes
            SET
                "status" = 'pending',
                stock_name = NULL,
                start_price = NULL,
                end_price = NULL,
                change = NULL,
                updated_at = NOW ()
            WHERE
                id = $1;
        "#;

    sqlx::query(query).bind(id).fetch_all(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn check_stock_change_pending_exist(
    state: &AppStateV2,
    payload: &StockRequest,
) -> Result<Option<StockChange>, AppError> {
    Ok(sqlx::query_as(
        "
        SELECT stock_no, start_date, end_date, stock_name, start_price, end_price, change
        FROM stock_changes
        WHERE stock_no = $1 AND start_date = $2 AND end_date = $3 AND status = 'pending'
        ",
    )
    .bind(&payload.stock_no)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .fetch_optional(state.get_pool())
    .await?)
}

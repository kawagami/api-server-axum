use crate::{
    errors::{AppError, RequestError},
    state::AppStateV2,
    structs::stocks::{Stock, StockChange, StockRequest},
};

pub async fn get_codes(state: &AppStateV2) -> Result<Vec<Stock>, AppError> {
    let client = state.get_http_client();
    let url = "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_AVG_ALL";

    client
        .get(url)
        .send()
        .await?
        .json::<Vec<Stock>>()
        .await
        .map_err(AppError::from)
}

pub async fn save_codes(state: &AppStateV2, stocks: &[Stock]) -> Result<usize, AppError> {
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

pub async fn save_request(
    state: &AppStateV2,
    stock_no: &str,
    start_date: &str,
    end_date: &str,
) -> Result<(), AppError> {
    let pool = state.get_pool();
    let query = "
        INSERT INTO stock_changes (stock_no, start_date, end_date, status, created_at, updated_at)
        VALUES ($1, $2, $3, 'pending', now(), now())
    ";
    sqlx::query(query)
        .bind(stock_no)
        .bind(start_date)
        .bind(end_date)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_all_stock_changes(state: &AppStateV2) -> Result<Vec<StockChange>, AppError> {
    let pool = state.get_pool();
    let query = "SELECT stock_no, start_date, end_date, stock_name, start_price, end_price, change FROM stock_changes";
    let requests = sqlx::query_as::<_, StockChange>(query)
        .fetch_all(pool)
        .await?;

    Ok(requests)
}

/// 使用 fastapi 獲取股票期間差異
pub async fn get_stock_change_info(
    state: &AppStateV2,
    stock_form: &StockRequest,
) -> Result<StockChange, AppError> {
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

    Ok(response.json::<StockChange>().await?)
}

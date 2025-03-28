use crate::state::AppStateV2;
use crate::{errors::AppError, routes::auth};
use axum::{extract::State, middleware, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
        .route("/get_codes", get(get_codes))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stock {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ClosingPrice")]
    pub closing_price: String,
    #[serde(rename = "MonthlyAveragePrice")]
    pub monthly_average_price: String,
}

pub async fn get_codes(State(state): State<AppStateV2>) -> Result<Json<usize>, AppError> {
    let client = state.get_http_client();
    let url = "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_AVG_ALL";

    let response = client.get(url).send().await?.json::<Vec<Stock>>().await?;

    let count = save_codes(&state, &response).await?;

    Ok(Json(count))
}

async fn save_codes(state: &AppStateV2, stocks: &[Stock]) -> Result<usize, AppError> {
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

    tx.commit().await?; // 提交交易
    Ok(stocks.len()) // 回傳插入的筆數
}

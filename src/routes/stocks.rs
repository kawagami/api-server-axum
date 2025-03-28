use crate::repositories::stocks;
use crate::state::AppStateV2;
use crate::structs::stocks::{StockChange, StockRequest};
use crate::{errors::AppError, routes::auth};
use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
        .route("/get_codes", get(get_codes))
        .route("/create_stock_change", post(create_stock_change))
        .route("/get_all_stock_changes", get(get_all_stock_changes))
        .route("/get_stock_change_info", post(get_stock_change_info))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
}

pub async fn get_codes(State(state): State<AppStateV2>) -> Result<Json<usize>, AppError> {
    let response = stocks::get_codes(&state).await?;

    let count = stocks::save_codes(&state, &response).await?;

    Ok(Json(count))
}

// 新增 pending 的等待查詢的股票代號 & 時間區間
pub async fn create_stock_change(
    State(state): State<AppStateV2>,
    Json(payload): Json<StockRequest>,
) -> Result<Json<String>, AppError> {
    stocks::save_request(
        &state,
        &payload.stock_no,
        &payload.start_date,
        &payload.end_date,
    )
    .await?;

    Ok(Json("Request saved successfully".to_string()))
}

pub async fn get_all_stock_changes(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockChange>>, AppError> {
    Ok(Json(stocks::get_all_stock_changes(&state).await?))
}

// 
pub async fn get_stock_change_info(
    State(state): State<AppStateV2>,
    Json(payload): Json<StockRequest>,
) -> Result<Json<StockChange>, AppError> {
    let pool = state.get_pool();

    // 1️⃣ 先查詢資料庫是否已有該筆資料
    let existing_info = sqlx::query_as(
        "
        SELECT stock_no, start_date, end_date, stock_name, start_price, end_price, change
        FROM stock_changes
        WHERE stock_no = $1 AND start_date = $2 AND end_date = $3 AND status = 'completed'
        ",
    )
    .bind(&payload.stock_no)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .fetch_optional(pool)
    .await?;

    // 2️⃣ 如果資料已存在，直接返回
    if let Some(info) = existing_info {
        return Ok(Json(info));
    }

    // 3️⃣ 沒有資料的話，向 FastAPI 查詢
    let info = stocks::get_stock_change_info(&state, &payload).await?;

    // 4️⃣ 更新新查詢到的資料到資料庫
    sqlx::query(
        "
        INSERT INTO stock_changes (stock_no, stock_name, start_date, start_price, end_date, end_price, change, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'completed', now(), now())
        ON CONFLICT (stock_no, start_date, end_date) DO UPDATE 
        SET status = 'completed',
            stock_name = EXCLUDED.stock_name,
            start_price = EXCLUDED.start_price,
            end_price = EXCLUDED.end_price,
            change = EXCLUDED.change,
            updated_at = now()

        ")
    .bind(&info.stock_no) 
    .bind(&info.stock_name)
    .bind(&info.start_date)
    .bind(&info.start_price)
    .bind(&info.end_date)
    .bind(&info.end_price)
    .bind(&info.change)
    .execute(pool)
    .await?;

    Ok(Json(info))
}

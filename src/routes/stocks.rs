use crate::errors::RequestError;
use crate::repositories::stocks;
use crate::services::stocks::parse_document;
use crate::state::AppStateV2;
use crate::structs::stocks::{
    BuybackDuration, Conditions, StockChange, StockChangeId, StockChangeWithoutId, StockRequest,
};
use crate::{errors::AppError, routes::auth};
use axum::{
    extract::{Query, State},
    middleware,
    routing::{get, patch, post},
    Json, Router,
};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
        .route("/get_codes", get(get_codes))
        .route("/new_pending_stock_change", post(new_pending_stock_change))
        .route("/get_all_stock_changes", get(get_all_stock_changes))
        .route("/get_stock_change_info", post(get_stock_change_info))
        .route("/buyback_stock_record", post(buyback_stock_record))
        .route("/get_all_failed", get(get_all_failed))
        .route(
            "/update_stock_change_pending",
            patch(update_stock_change_pending),
        )
        .route(
            "/update_one_stock_change_pending",
            patch(update_one_stock_change_pending),
        )
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
pub async fn new_pending_stock_change(
    State(state): State<AppStateV2>,
    Json(payload): Json<StockRequest>,
) -> Result<Json<StockChange>, AppError> {
    let pool = state.get_pool();

    // 先查詢資料庫是否已有該筆資料
    let existing_info = sqlx::query_as(
        "
        SELECT stock_no, start_date, end_date, stock_name, start_price, end_price, change
        FROM stock_changes
        WHERE stock_no = $1 AND start_date = $2 AND end_date = $3 AND status = 'pending'
        ",
    )
    .bind(&payload.stock_no)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .fetch_optional(pool)
    .await?;

    // 如果資料已存在，直接返回
    if let Some(info) = existing_info {
        return Ok(Json(info));
    }

    // 沒資料的話加入排程
    stocks::save_request(
        &state,
        &payload.stock_no,
        &payload.start_date,
        &payload.end_date,
    )
    .await?;

    Ok(Json(StockChange::default()))
}

pub async fn get_all_stock_changes(
    State(state): State<AppStateV2>,
    Query(payload): Query<Conditions>,
) -> Result<Json<Vec<StockChange>>, AppError> {
    Ok(Json(stocks::get_all_stock_changes(&state, payload).await?))
}

//
pub async fn get_stock_change_info(
    State(state): State<AppStateV2>,
    Json(payload): Json<StockRequest>,
) -> Result<Json<StockChangeWithoutId>, AppError> {
    // 先查詢資料庫是否已有該筆資料
    let existing_info = stocks::get_existing_stock_change(&state, &payload).await?;

    // 如果資料已存在，直接返回
    if let Some(info) = existing_info {
        return Ok(Json(info));
    }

    // 沒有資料的話，向 FastAPI 查詢
    let info = stocks::get_stock_change_info(&state, &payload).await?;

    // 更新新查詢到的資料到資料庫
    let _ = stocks::upsert_stock_change(&state, &info).await;

    Ok(Json(info))
}

//依照 input 的時間區間抓資料
pub async fn buyback_stock_record(
    State(state): State<AppStateV2>,
    Json(payload): Json<BuybackDuration>,
) -> Result<Json<Vec<StockRequest>>, AppError> {
    // Create HTTP client
    let client = state.get_http_client();

    // Prepare form data
    let form_data = form_urlencoded::Serializer::new(String::new())
        .append_pair("encodeURIComponent", "1")
        .append_pair("step", "1")
        .append_pair("firstin", "1")
        .append_pair("off", "1")
        .append_pair("TYPEK", "sii")
        .append_pair("d1", &payload.start_date)
        .append_pair("d2", &payload.end_date)
        .append_pair("RD", "1")
        .finish();

    // Send POST request to get the data
    let response = client
        .post("https://mopsov.twse.com.tw/mops/web/ajax_t35sc09")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_data)
        .send()
        .await?;

    // Check if request was successful
    if !response.status().is_success() {
        return Err(AppError::RequestError(RequestError::InvalidContent(
            "取資料失敗".to_string(),
        )));
    }

    // Parse the HTML content
    let html = response.text().await?;

    let records = parse_document(html);

    // 批次寫入取得的資料
    let _ = stocks::insert_stock_data_batch(&state, &records).await?;

    Ok(Json(records))
}

pub async fn get_all_failed(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockRequest>>, AppError> {
    Ok(Json(stocks::get_all_failed(&state).await?))
}

pub async fn update_stock_change_pending(
    State(state): State<AppStateV2>,
) -> Result<Json<()>, AppError> {
    Ok(Json(stocks::update_stock_change_pending(&state).await?))
}

// 將有資料的 stock_change 改成 pending
pub async fn update_one_stock_change_pending(
    State(state): State<AppStateV2>,
    Json(payload): Json<StockChangeId>,
) -> Result<Json<()>, AppError> {
    Ok(Json(
        stocks::update_one_stock_change_pending(&state, payload.id).await?,
    ))
}

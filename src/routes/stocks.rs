use crate::repositories::stocks;
use crate::services::stocks::{
    get_buyback_stock_raw_html_string, get_stock_day_avg, parse_buyback_stock_raw_html,
    parse_stock_day_avg_response,
};
use crate::state::AppStateV2;
use crate::structs::stocks::{
    BuybackDuration, Conditions, GetStockHistoryPriceRequest, NewStockClosingPrice, StockChange,
    StockChangeId, StockChangeWithoutId, StockClosingPrice, StockRequest,
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
        .route(
            "/fetch_and_save_stock_day_avg_all",
            get(fetch_and_save_stock_day_avg_all),
        )
        .route("/new_pending_stock_change", post(new_pending_stock_change))
        .route("/get_all_stock_changes", get(get_all_stock_changes))
        .route("/get_stock_change_info", post(get_stock_change_info))
        .route("/buyback_stock_record", post(buyback_stock_record))
        .route(
            "/reset_failed_stock_changes_to_pending",
            patch(reset_failed_stock_changes_to_pending),
        )
        .route(
            "/update_one_stock_change_pending",
            patch(update_one_stock_change_pending),
        )
        .route("/get_stock_history_price", get(get_stock_history_price))
        .route(
            "/get_all_stock_closing_prices",
            get(get_all_stock_closing_prices),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
}

// 打 openapi 取當天所有股票的 STOCK_DAY_AVG_ALL 資料
pub async fn fetch_and_save_stock_day_avg_all(
    State(state): State<AppStateV2>,
) -> Result<Json<usize>, AppError> {
    let response = stocks::fetch_stock_day_avg_all(&state).await?;

    let count = stocks::save_stock_day_avg_all(&state, &response).await?;

    Ok(Json(count))
}

// 新增 pending 的等待查詢的股票代號 & 時間區間
pub async fn new_pending_stock_change(
    State(state): State<AppStateV2>,
    Json(payload): Json<StockRequest>,
) -> Result<Json<StockChange>, AppError> {
    // 先查詢資料庫是否已有該筆資料
    let existing_info = stocks::check_stock_change_pending_exist(&state, &payload).await?;

    // 如果資料已存在，直接返回
    if let Some(info) = existing_info {
        return Ok(Json(info));
    }

    // 沒資料的話加入排程
    stocks::save_request(&state, &payload).await?;

    Ok(Json(StockChange::default()))
}

pub async fn get_all_stock_changes(
    State(state): State<AppStateV2>,
    Query(payload): Query<Conditions>,
) -> Result<Json<Vec<StockChange>>, AppError> {
    Ok(Json(stocks::get_all_stock_changes(&state, payload).await?))
}

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

// 依照 input 的時間區間抓資料
pub async fn buyback_stock_record(
    State(state): State<AppStateV2>,
    Json(payload): Json<BuybackDuration>,
) -> Result<Json<Vec<StockRequest>>, AppError> {
    let records = parse_buyback_stock_raw_html(
        get_buyback_stock_raw_html_string(
            state.get_http_client(),
            &payload.start_date,
            &payload.end_date,
        )
        .await?,
    );

    // 批次寫入取得的資料
    stocks::insert_stock_data_batch(&state, &records).await?;

    Ok(Json(records))
}

pub async fn reset_failed_stock_changes_to_pending(
    State(state): State<AppStateV2>,
) -> Result<Json<()>, AppError> {
    Ok(Json(
        stocks::reset_failed_stock_changes_to_pending(&state).await?,
    ))
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

// 打外部 API 取歷史收盤價
pub async fn get_stock_history_price(
    State(state): State<AppStateV2>,
    Query(payload): Query<GetStockHistoryPriceRequest>,
) -> Result<Json<Vec<NewStockClosingPrice>>, AppError> {
    // 先看資料庫有沒有查詢那個日期的資料
    let existing_prices = stocks::get_stock_closing_price(&state, &payload).await?;

    // 如果資料庫中已有該股票該日期的資料，則直接返回
    if !existing_prices.is_empty() {
        // 將 StockClosingPrice 轉換為 NewStockClosingPrice
        let result = existing_prices
            .into_iter()
            .map(|price| NewStockClosingPrice {
                stock_no: price.stock_no,
                date: price.date,
                close_price: price.close_price,
            })
            .collect::<Vec<NewStockClosingPrice>>();

        return Ok(Json(result));
    }

    // 沒有的話打外部 API 取歷史收盤價
    let new_stock_closing_prices = parse_stock_day_avg_response(
        get_stock_day_avg(state.get_http_client(), &payload.stock_no, &payload.date).await?,
        &payload.stock_no,
    );

    // 將歷史價寫進資料庫
    stocks::upsert_stock_closing_prices(&state, &new_stock_closing_prices).await?;

    Ok(Json(new_stock_closing_prices))
}

// 取資料庫中所有歷史收盤價
pub async fn get_all_stock_closing_prices(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockClosingPrice>>, AppError> {
    Ok(Json(stocks::get_all_stock_closing_prices(&state).await?))
}

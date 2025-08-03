use crate::{
    errors::AppError,
    repositories::stocks,
    routes::auth,
    services::stocks::{
        fetch_stock_price_for_date, get_buyback_stock_raw_html_string, get_stock_day_avg,
        parse_buyback_stock_raw_html, parse_stock_day_avg_response, round_to_n_decimal,
        stock_day_all_service,
    },
    state::AppStateV2,
    structs::stocks::{
        BuybackDuration, Conditions, GetStockDayAll, GetStockHistoryPriceRequest,
        NewStockClosingPrice, StartPriceFilter, StockBuybackInfo, StockBuybackMoreInfo,
        StockBuybackPeriod, StockChange, StockChangeId, StockChangeWithoutId, StockClosingPrice,
        StockClosingPriceResponse, StockRequest, StockStats,
    },
};
use axum::{
    extract::{Query, State},
    middleware,
    routing::{get, patch, post},
    Json, Router,
};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
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
        .route(
            "/fetch_stock_closing_price_pair_stats",
            get(fetch_stock_closing_price_pair_stats),
        )
        .route("/bulk_insert_stock_day_all", get(bulk_insert_stock_day_all))
        .route("/get_stock_day_all", get(get_stock_day_all))
        .route("/get_stock_buyback_periods", get(get_stock_buyback_periods))
        .route(
            "/get_unfinished_buyback_price_gap",
            get(get_unfinished_buyback_price_gap),
        )
        .route("/testing_api", get(testing_api))
        .route("/testing_api2", get(testing_api2))
        .route(
            "/get_stock_buyback_periods_v2",
            get(get_stock_buyback_periods_v2),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
}

/// 新增 pending 的等待查詢的股票代號 & 時間區間
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

/// 依照 input 的時間區間抓資料
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

/// 打外部 API 取歷史收盤價 寫進 stock_closing_prices table
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

    // 將歷史價寫進資料庫 stock_closing_prices 只記錄特定股票在特定日的收盤價
    stocks::upsert_stock_closing_prices(&state, &new_stock_closing_prices).await?;

    Ok(Json(new_stock_closing_prices))
}

/// 取資料庫中所有歷史收盤價
pub async fn get_all_stock_closing_prices(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockClosingPrice>>, AppError> {
    Ok(Json(stocks::get_all_stock_closing_prices(&state).await?))
}

/// 打外部 API 取 start_date & end_date 的歷史收盤價 增加額外統計資訊
pub async fn fetch_stock_closing_price_pair_stats(
    State(state): State<AppStateV2>,
    Query(payload): Query<StockRequest>,
) -> Result<Json<StockClosingPriceResponse>, AppError> {
    let (start_price, end_price) = tokio::try_join!(
        fetch_stock_price_for_date(&state, &payload.stock_no, &payload.start_date),
        fetch_stock_price_for_date(&state, &payload.stock_no, &payload.end_date)
    )?;

    let price_diff = round_to_n_decimal(end_price.close_price - start_price.close_price, 2);
    let raw_percent_change = if start_price.close_price != 0.0 {
        (price_diff / start_price.close_price) * 100.0
    } else {
        0.0
    };
    let percent_change = round_to_n_decimal(raw_percent_change, 2);
    let is_increase = price_diff > 0.0;

    let day_span = (end_price.date - start_price.date).num_days();

    let stats = StockStats {
        price_diff,
        percent_change,
        is_increase,
        day_span,
    };

    let response = StockClosingPriceResponse {
        prices: (start_price, end_price),
        stats,
    };

    Ok(Json(response))
}

/// Job StockDayAllJob 會排程執行 可考慮移除
pub async fn bulk_insert_stock_day_all(
    State(state): State<AppStateV2>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    stock_day_all_service(&state).await?;

    Ok(Json("成功"))
}

pub async fn get_stock_day_all(
    State(state): State<AppStateV2>,
    Query(payload): Query<GetStockDayAll>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let response =
        stocks::get_stock_day_all(&state, payload.stock_code, payload.trade_date).await?;

    Ok(Json(response))
}

/// 依照 input 的時間區間抓資料
/// 只記錄庫藏股起訖時間資訊
pub async fn get_stock_buyback_periods(
    State(state): State<AppStateV2>,
    Json(payload): Json<BuybackDuration>,
) -> Result<Json<Vec<StockRequest>>, AppError> {
    // 先取得庫藏股頁面 raw string => 解析成 Vec<StockRequest> 資料
    let records = parse_buyback_stock_raw_html(
        get_buyback_stock_raw_html_string(
            state.get_http_client(),
            &payload.start_date,
            &payload.end_date,
        )
        .await?,
    );

    // 批次寫入取得的資料
    stocks::bulk_insert_stock_buyback_periods(&state, &records).await?;

    Ok(Json(records))
}

/// 取得未到結束日的庫藏股起始日到現在的價格差距 & 資訊
pub async fn get_unfinished_buyback_price_gap(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockBuybackMoreInfo>>, AppError> {
    Ok(Json(stocks::get_active_buyback_prices(&state).await?))
}

/// 紀錄執行計畫中的 API
pub async fn testing_api(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockBuybackInfo>>, AppError> {
    let data = stocks::get_active_buyback_prices_v4(&state, StartPriceFilter::All).await?;

    Ok(Json(data))
}

pub async fn testing_api2(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockBuybackInfo>>, AppError> {
    let data = stocks::get_active_buyback_prices_v4(&state, StartPriceFilter::ExistsOnly).await?;

    Ok(Json(data))
}

/// 取得 DB 資料中的紀錄
pub async fn get_stock_buyback_periods_v2(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<StockBuybackPeriod>>, AppError> {
    let data = stocks::get_stock_buyback_periods(&state).await?;

    Ok(Json(data))
}

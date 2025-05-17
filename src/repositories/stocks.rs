use crate::{
    errors::{AppError, RequestError},
    state::AppStateV2,
    structs::stocks::{
        Conditions, GetStockHistoryPriceRequest, NewStockClosingPrice, Stock, StockChange,
        StockChangeWithoutId, StockClosingPrice, StockDayAll, StockRequest,
    },
};
use chrono::NaiveDate;
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

pub async fn save_stock_day_avg_all(
    state: &AppStateV2,
    stocks: &[Stock],
) -> Result<usize, AppError> {
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

// 預計改用 rust 實現的方式
// 使用 fastapi 獲取股票期間差異
pub async fn get_stock_change_info(
    state: &AppStateV2,
    stock_form: &StockRequest,
) -> Result<StockChangeWithoutId, AppError> {
    let client = state.get_http_client();

    // 此 API 在 fastapi 打外部 API 取兩個時間點的歷史價格整理後返回
    let url = format!("{}{}", state.get_fastapi_upload_host(), "/stock-change");

    // 把這部分改成 stocks service 完成
    let response = client
        .post(url)
        .json(&stock_form)
        .send()
        .await
        .map_err(|err| AppError::ConnectionError(err.into()))?;

    // 先檢查狀態碼
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
            AND TO_DATE(
                (CAST((CAST(end_date AS TEXT)::INT + 19110000) AS TEXT)), 
                'YYYYMMDD'
            ) <= CURRENT_DATE
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

pub async fn get_all_stock_closing_prices(
    state: &AppStateV2,
) -> Result<Vec<StockClosingPrice>, AppError> {
    let mut query = QueryBuilder::new(
        r#"
        SELECT
            *
        FROM
            stock_closing_prices s
        WHERE 1=1
    "#,
    );

    // // Add status condition if it exists
    // if let Some(status) = &conditions.status {
    //     query.push(" AND s.status = ");
    //     query.push_bind(status);
    // }

    // Add the ordering at the end
    query.push(" ORDER BY s.date DESC");

    // Execute the query with the arguments
    let requests: Vec<StockClosingPrice> =
        query.build_query_as().fetch_all(state.get_pool()).await?;

    Ok(requests)
}

pub async fn get_stock_closing_price(
    state: &AppStateV2,
    query: &GetStockHistoryPriceRequest,
) -> Result<Vec<StockClosingPrice>, AppError> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT
            *
        FROM
            stock_closing_prices s
        WHERE 1=1
    "#,
    );

    // 添加股票編號條件 (必填)
    query_builder.push(" AND s.stock_no = ");
    query_builder.push_bind(&query.stock_no);

    // 添加日期條件，將 YYYYMMDD 格式的字串轉換為日期格式
    query_builder.push(" AND s.date = ");
    query_builder.push(" TO_DATE(");
    query_builder.push_bind(&query.date);
    query_builder.push(", 'YYYYMMDD')");

    // 執行查詢並獲取結果
    let results: Vec<StockClosingPrice> = query_builder
        .build_query_as()
        .fetch_all(state.get_pool())
        .await?;

    Ok(results)
}

pub async fn upsert_stock_closing_prices(
    state: &AppStateV2,
    data: &Vec<NewStockClosingPrice>,
) -> Result<(), AppError> {
    if data.is_empty() {
        return Ok(()); // 無資料可寫入
    }

    let now = chrono::Utc::now().naive_utc(); // 統一時間

    let mut query_builder = QueryBuilder::new(
        "INSERT INTO stock_closing_prices (stock_no, date, close_price, created_at, updated_at) ",
    );

    // 插入多筆
    query_builder.push_values(data.iter(), |mut b, row| {
        b.push_bind(&row.stock_no)
            .push_bind(row.date)
            .push_bind(row.close_price)
            .push_bind(now)
            .push_bind(now);
    });

    // ON CONFLICT 更新條件
    query_builder.push(
        " ON CONFLICT (stock_no, date) DO UPDATE SET close_price = EXCLUDED.close_price, updated_at = EXCLUDED.updated_at",
    );

    let query = query_builder.build();

    query.execute(state.get_pool()).await?;

    Ok(())
}

pub async fn get_stock_closing_prices_by_date_range(
    state: &AppStateV2,
    stock_no: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<NewStockClosingPrice>, AppError> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT
            *
        FROM
            stock_closing_prices s
        WHERE 1=1
    "#,
    );

    // 添加股票編號條件 (必填)
    query_builder.push(" AND s.stock_no = ");
    query_builder.push_bind(stock_no);

    // 添加日期條件，將 YYYYMMDD 格式的字串轉換為日期格式
    query_builder.push(" AND s.date BETWEEN ");
    query_builder.push(" TO_DATE(");
    query_builder.push_bind(start_date);
    query_builder.push(", 'YYYYMMDD')");
    query_builder.push(" AND TO_DATE(");
    query_builder.push_bind(end_date);
    query_builder.push(", 'YYYYMMDD')");

    // 執行查詢並獲取結果
    let results: Vec<NewStockClosingPrice> = query_builder
        .build_query_as()
        .fetch_all(state.get_pool())
        .await?;

    Ok(results)
}

// let stock_code: Option<&str> = Some("00645");
// let trade_date: Option<NaiveDate> = None;

pub async fn get_stock_day_all(
    state: &AppStateV2,
    stock_code: Option<String>,
    trade_date: Option<NaiveDate>,
) -> Result<Vec<StockDayAll>, AppError> {
    let mut builder = QueryBuilder::new("SELECT * FROM stock_day_all");

    let mut has_where = false;

    if stock_code.is_some() || trade_date.is_some() {
        builder.push(" WHERE ");
    }

    if let Some(code) = stock_code {
        builder.push("stock_code = ").push_bind(code);
        has_where = true;
    }

    if let Some(date) = trade_date {
        if has_where {
            builder.push(" AND ");
        }
        builder.push("trade_date = ").push_bind(date);
    }

    builder.push(" ORDER BY trade_date DESC");

    let query = builder.build_query_as::<StockDayAll>();
    let results = query.fetch_all(state.get_pool()).await?;

    Ok(results)
}

pub async fn bulk_insert_stock_buyback_periods(
    state: &AppStateV2,
    stocks: &[StockRequest],
) -> Result<usize, AppError> {
    let mut tx = state.get_pool().begin().await?;

    let query = "
        INSERT INTO stock_buyback_periods (stock_no, start_date, end_date)
        SELECT * FROM UNNEST(
            $1::text[], $2::date[], $3::date[]
        )
        ON CONFLICT (stock_no, start_date, end_date) DO NOTHING;
    ";

    let stock_nos: Vec<&str> = stocks.iter().map(|s| s.stock_no.as_str()).collect();

    // 轉換民國年月日為 NaiveDate
    let start_dates: Result<Vec<NaiveDate>, AppError> = stocks
        .iter()
        .map(|s| roc_date_to_naive_date(s.start_date.as_str()))
        .collect();
    let start_dates = start_dates?;

    let end_dates: Result<Vec<NaiveDate>, AppError> = stocks
        .iter()
        .map(|s| roc_date_to_naive_date(s.end_date.as_str()))
        .collect();
    let end_dates = end_dates?;

    sqlx::query(query)
        .bind(&stock_nos)
        .bind(&start_dates)
        .bind(&end_dates)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(stocks.len())
}

// 將民國年月日轉換為 NaiveDate
fn roc_date_to_naive_date(roc_date: &str) -> Result<NaiveDate, AppError> {
    if roc_date.len() != 7 {
        return Err(RequestError::InvalidContent(format!("無效的民國日期格式")).into());
    }

    // 取出民國年、月、日
    let roc_year = &roc_date[0..3];
    let month = &roc_date[3..5];
    let day = &roc_date[5..7];

    // 轉換為數字
    let roc_year: i32 = roc_year.parse().unwrap();
    let month: u32 = month.parse().unwrap();
    let day: u32 = day.parse().unwrap();

    // 民國年轉換為西元年 (民國年+1911=西元年)
    let gregorian_year = roc_year + 1911;

    // 創建 NaiveDate
    chrono::NaiveDate::from_ymd_opt(gregorian_year, month, day)
        .ok_or_else(|| RequestError::InvalidContent(format!("創建 NaiveDate fail")).into())
}

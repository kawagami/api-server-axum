use crate::{
    errors::AppError,
    state::AppState,
    structs::stocks::{
        Conditions, StockChange, StockChangePaginatedResponse, StockChangeRef,
    },
};
use sqlx::{QueryBuilder, Row};

pub async fn get_all_stock_changes(
    state: &AppState,
    conditions: Conditions,
) -> Result<StockChangePaginatedResponse, AppError> {
    let mut count_query = QueryBuilder::new("SELECT COUNT(*) FROM stock_changes s WHERE 1=1");
    let mut data_query = QueryBuilder::new("SELECT * FROM stock_changes s WHERE 1=1");

    if let Some(status) = &conditions.status {
        count_query.push(" AND s.status = ");
        count_query.push_bind(status);
        data_query.push(" AND s.status = ");
        data_query.push_bind(status);
    }

    data_query.push(" ORDER BY s.start_date DESC LIMIT ");
    data_query.push_bind(conditions.limit);
    data_query.push(" OFFSET ");
    data_query.push_bind(conditions.offset);

    let total: i64 = count_query
        .build()
        .fetch_one(state.get_pool())
        .await?
        .get(0);

    let data = data_query
        .build_query_as()
        .fetch_all(state.get_pool())
        .await?;

    Ok(StockChangePaginatedResponse { data, total })
}

pub async fn get_one_pending_stock_change(
    state: &AppState,
) -> Result<Option<StockChangeRef>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT stock_no, start_date, end_date
        FROM stock_changes
        WHERE status = 'pending'
            AND end_date <= CURRENT_DATE
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(state.get_pool())
    .await?;

    if let Some(row) = row {
        Ok(Some(StockChangeRef {
            stock_no: row.get("stock_no"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn upsert_stock_change(
    state: &AppState,
    info: &StockChange,
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
    .bind(info.start_date)
    .bind(&info.start_price)
    .bind(info.end_date)
    .bind(&info.end_price)
    .bind(&info.change)
    .execute(state.get_pool())
    .await?;

    Ok(())
}

pub async fn update_stock_change_failed(
    state: &AppState,
    stock: &StockChangeRef,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE stock_changes SET updated_at = NOW(), status = 'failed'
        WHERE stock_no = $1 AND start_date = $2 AND end_date = $3",
    )
    .bind(&stock.stock_no)
    .bind(stock.start_date)
    .bind(stock.end_date)
    .execute(state.get_pool())
    .await?;

    Ok(())
}

pub async fn sync_buyback_periods_to_pending(state: &AppState) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO stock_changes (stock_no, start_date, end_date)
        SELECT bp.stock_no, bp.start_date, bp.end_date
        FROM stock_buyback_periods bp
        ON CONFLICT (stock_no, start_date, end_date) DO NOTHING
        "#,
    )
    .execute(state.get_pool())
    .await?;

    Ok(result.rows_affected())
}

pub async fn update_one_stock_change_pending(state: &AppState, id: i32) -> Result<(), AppError> {
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

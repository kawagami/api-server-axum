use crate::{errors::AppError, state::AppStateV2, structs::stocks::StockDayAll};
use chrono::NaiveDate;
use sqlx::QueryBuilder;

pub async fn get_stock_day_all(
    state: &AppStateV2,
    stock_code: Option<String>,
    trade_date: Option<NaiveDate>,
    limit: i64,
    offset: i64,
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

    builder.push(" ORDER BY trade_date DESC, stock_code ASC");
    builder.push(" LIMIT ").push_bind(limit);
    builder.push(" OFFSET ").push_bind(offset);

    Ok(builder.build_query_as::<StockDayAll>().fetch_all(state.get_pool()).await?)
}

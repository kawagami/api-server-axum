use crate::{
    errors::AppError,
    state::AppState,
    structs::stocks::NewStockClosingPrice,
};
use sqlx::QueryBuilder;

pub async fn upsert_stock_closing_prices(
    state: &AppState,
    data: &Vec<NewStockClosingPrice>,
) -> Result<(), AppError> {
    if data.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now().naive_utc();

    let mut qb = QueryBuilder::new(
        "INSERT INTO stock_closing_prices (stock_no, date, close_price, created_at, updated_at) ",
    );

    qb.push_values(data.iter(), |mut b, row| {
        b.push_bind(&row.stock_no)
            .push_bind(row.date)
            .push_bind(row.close_price)
            .push_bind(now)
            .push_bind(now);
    });

    qb.push(" ON CONFLICT (stock_no, date) DO UPDATE SET close_price = EXCLUDED.close_price, updated_at = EXCLUDED.updated_at");
    qb.build().execute(state.get_pool()).await?;

    Ok(())
}

pub async fn get_stock_closing_prices_by_date_range(
    state: &AppState,
    stock_no: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<NewStockClosingPrice>, AppError> {
    let mut qb = QueryBuilder::new("SELECT * FROM stock_closing_prices s WHERE 1=1");

    qb.push(" AND s.stock_no = ");
    qb.push_bind(stock_no);
    qb.push(" AND s.date BETWEEN TO_DATE(");
    qb.push_bind(start_date);
    qb.push(", 'YYYYMMDD') AND TO_DATE(");
    qb.push_bind(end_date);
    qb.push(", 'YYYYMMDD')");

    Ok(qb.build_query_as().fetch_all(state.get_pool()).await?)
}

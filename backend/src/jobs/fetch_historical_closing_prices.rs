use crate::{
    repositories::stocks::{
        get_buybacks_missing_start_price, mark_buyback_start_price_unavailable, upsert_stock_closing_prices,
    },
    services::stocks::{get_stock_day_avg_or_none, parse_stock_day_avg_response},
    state::AppState,
};

pub async fn run(state: AppState) {
    let pool = state.get_pool();
    let client = state.get_http_client();

    let no_start_price_data = match get_buybacks_missing_start_price(pool).await {
        Ok(data) => data,
        Err(e) => {
            // 同檔其他 error 都帶得出「哪支股票哪一天」，這行原本只有一句裸訊息
            tracing::error!("get_buybacks_missing_start_price fail: {}", e);
            return;
        }
    };

    // Take the oldest entry first; one per minute to avoid TWSE rate limiting
    if let Some(data) = no_start_price_data.into_iter().next() {
        let avg_response = match get_stock_day_avg_or_none(client, &data.stock_no, data.start_date).await {
            Ok(Some(r)) => r,
            // TWSE 確認這個月沒有（上櫃 / 轉上市前）：不記下來的話這筆每分鐘被重撿，
            // 排在後面的期間全部補不到。其他錯誤可能是暫時的，走下面那支、下一分鐘重試。
            Ok(None) => {
                tracing::warn!(
                    "TWSE 查無起始日收盤價，標記不再回補 stock_no={} start_date={}（上櫃 / 轉上市前，系統只接 TWSE）",
                    data.stock_no,
                    data.start_date
                );
                if let Err(e) = mark_buyback_start_price_unavailable(pool, &data.stock_no, data.start_date).await {
                    tracing::error!("mark_buyback_start_price_unavailable fail stock_no={}: {:?}", data.stock_no, e);
                }
                return;
            }
            Err(e) => {
                // `{:?}`：AppError 的 Display 只有分類字串，原因在 source 裡
                tracing::error!("get_stock_day_avg fail stock_no={} date={}: {:?}", data.stock_no, data.start_date, e);
                return;
            }
        };

        let new_stock_closing_prices = parse_stock_day_avg_response(avg_response, &data.stock_no);

        if let Err(e) = upsert_stock_closing_prices(pool, &new_stock_closing_prices).await {
            tracing::error!("upsert_stock_closing_prices fail stock_no={} date={}: {}", data.stock_no, data.start_date, e);
            return;
        }

        tracing::info!("{} {} upsert_stock_closing_prices success", data.stock_no, data.start_date);
    }
}

use crate::{
    errors::{AppError, RequestError},
    repositories::stocks::{
        get_one_pending_stock_change, update_stock_change_failed, upsert_stock_change,
    },
    services::stocks::get_stock_change_info,
    state::AppState,
    structs::ws::WsEvent,
};

pub async fn run(state: AppState) {
    let pool = state.get_pool();
    let client = state.get_http_client();

    let pending_stock = match get_one_pending_stock_change(pool).await {
        Ok(Some(stock)) => stock,
        Ok(None) => return,
        Err(err) => {
            tracing::error!("failed to fetch pending stock change: {:?}", err);
            return;
        }
    };

    let stock_info = match get_stock_change_info(pool, client, &pending_stock).await {
        Ok(info) => info,
        Err(err) => {
            let is_data_error =
                matches!(&err, AppError::RequestError(RequestError::InvalidContent(_)));
            if is_data_error {
                if let Err(e) = update_stock_change_failed(pool, &pending_stock).await {
                    tracing::error!("update_stock_change_failed stock_no={}: {:?}", pending_stock.stock_no, e);
                }
                state.broadcast(
                    WsEvent::StockFailed,
                    serde_json::json!({ "stock_no": pending_stock.stock_no }),
                );
            } else {
                tracing::error!(
                    "transient error for stock_no={}: {:?}",
                    pending_stock.stock_no,
                    err
                );
            }
            return;
        }
    };

    if let Err(err) = upsert_stock_change(pool, &stock_info).await {
        tracing::error!("failed to upsert stock_change stock_no={}: {:?}", stock_info.stock_no, err);
        return;
    }

    tracing::info!(
        "stock_change completed stock_no={} change={:?}",
        stock_info.stock_no,
        stock_info.change
    );
    state.broadcast(
        WsEvent::StockCompleted,
        serde_json::json!({
            "stock_no": stock_info.stock_no,
            "stock_name": stock_info.stock_name,
            "start_date": stock_info.start_date,
            "end_date": stock_info.end_date,
            "change": stock_info.change,
        }),
    );
}

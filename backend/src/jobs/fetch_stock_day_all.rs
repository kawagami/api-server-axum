use crate::{services::stocks::stock_day_all_service, state::AppState};

pub async fn run(state: AppState) {
    let pool = state.get_pool().clone();
    let client = state.get_http_client().clone();
    super::run_with_retries(
        "stock_day_all_service",
        3,
        std::time::Duration::from_secs(3600),
        || stock_day_all_service(&pool, &client),
    )
    .await;
}

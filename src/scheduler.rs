use crate::{
    jobs::{
        fetch_historical_closing_prices::FetchHistoricalClosingPricesJob,
        cleanup_unused_images::CleanupUnusedImagesJob,
        fetch_notes::FetchNotesJob,
        fetch_buyback_periods::FetchBuybackPeriodsJob,
        fetch_stock_day_all::FetchStockDayAllJob,
        consume_pending_stock_change::ConsumePendingStockChangeJob,
        sync_buyback_to_pending::SyncBuybackToPendingJob,
    },
    state::AppState,
    structs::jobs::AppJob,
};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn initialize_scheduler(state: AppState) {
    let scheduler = JobScheduler::new().await.expect("failed to create scheduler");

    add_job(&scheduler, state.clone(), CleanupUnusedImagesJob).await;
    add_job(&scheduler, state.clone(), FetchStockDayAllJob).await;
    add_job(&scheduler, state.clone(), FetchBuybackPeriodsJob).await;
    add_job(&scheduler, state.clone(), FetchNotesJob).await;
    add_job(&scheduler, state.clone(), FetchHistoricalClosingPricesJob).await;
    add_job(&scheduler, state.clone(), ConsumePendingStockChangeJob).await;
    add_job(&scheduler, state.clone(), SyncBuybackToPendingJob).await;

    scheduler.start().await.expect("failed to start scheduler");
}

async fn add_job<J: AppJob + Send + Sync + 'static>(
    scheduler: &JobScheduler,
    state: AppState,
    job_instance: J,
) {
    let expr = job_instance.cron_expression().to_string();
    let job_arc = Arc::new(job_instance);

    let job = match Job::new_async(expr.as_str(), move |_uuid, _l| {
        let state = state.clone();
        let job = job_arc.clone();
        Box::pin(async move { job.run(state).await })
    }) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!("failed to create job ({}): {:?}", expr, e);
            return;
        }
    };

    if let Err(e) = scheduler.add(job).await {
        tracing::error!("failed to add job ({}): {:?}", expr, e);
    }
}

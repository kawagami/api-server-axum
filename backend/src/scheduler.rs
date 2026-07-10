use crate::{state::AppState, structs::jobs::AppJob};
use std::sync::Arc;
use tokio_cron_scheduler::{Job as CronJob, JobScheduler};

pub async fn initialize_scheduler(state: AppState) {
    let scheduler = JobScheduler::new().await.expect("failed to create scheduler");

    for job in [
        AppJob::CleanupExpiredTorrents,
        AppJob::CleanupUnusedImages,
        AppJob::FetchStockDayAll,
        AppJob::FetchBuybackPeriods,
        AppJob::FetchGovTenders,
        AppJob::FetchHistoricalClosingPrices,
        AppJob::ConsumePendingStockChange,
        AppJob::SyncBuybackToPending,
        AppJob::CheckInvoiceLottery,
        AppJob::CheckLottoWins,
        AppJob::AggregateVisitors,
    ] {
        add_job(&scheduler, state.clone(), job).await;
    }

    scheduler.start().await.expect("failed to start scheduler");
}

async fn add_job(scheduler: &JobScheduler, state: AppState, job: AppJob) {
    let expr = job.cron_expression().to_string();
    let job = Arc::new(job);
    // 防重疊：上一輪還沒跑完就跳過本輪
    let running = Arc::new(tokio::sync::Mutex::new(()));

    let cron_job = match CronJob::new_async(expr.as_str(), move |_uuid, _l| {
        let state = state.clone();
        let job = job.clone();
        let running = running.clone();
        Box::pin(async move {
            let Ok(_guard) = running.try_lock() else {
                tracing::warn!("job {} still running, skip this tick", job.name());
                return;
            };
            job.run(state).await
        })
    }) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!("failed to create job ({}): {:?}", expr, e);
            return;
        }
    };

    if let Err(e) = scheduler.add(cron_job).await {
        tracing::error!("failed to add job ({}): {:?}", expr, e);
    }
}

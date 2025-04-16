use crate::{
    jobs::{
        blogs::ActiveImageJob, example::ExampleJob, notes::FetchNotesJob,
        stocks::ConsumePendingStockChangeJob,
    },
    state::AppStateV2,
    structs::jobs::AppJob,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn initialize_scheduler(state: AppStateV2) -> Arc<Mutex<JobScheduler>> {
    let scheduler = Arc::new(Mutex::new(JobScheduler::new().await.unwrap()));

    add_job_if_enabled(scheduler.clone(), state.clone(), ExampleJob).await;
    add_job_if_enabled(scheduler.clone(), state.clone(), FetchNotesJob).await;
    add_job_if_enabled(scheduler.clone(), state.clone(), ActiveImageJob).await;
    add_job_if_enabled(
        scheduler.clone(),
        state.clone(),
        ConsumePendingStockChangeJob,
    )
    .await;

    scheduler.lock().await.start().await.unwrap();
    scheduler
}

/// 預設會執行 檢查是否有設定不執行
async fn add_job_if_enabled<J: AppJob + Clone + Send + Sync + 'static>(
    scheduler: Arc<Mutex<JobScheduler>>,
    state: AppStateV2,
    job_instance: J,
) {
    if job_instance.enabled() {
        add_job_to_scheduler(scheduler, state, job_instance).await;
    }
}

async fn add_job_to_scheduler<J: AppJob + Clone + Send + Sync + 'static>(
    scheduler: Arc<Mutex<JobScheduler>>,
    state: AppStateV2,
    job_instance: J,
) {
    let job = Job::new_async(job_instance.clone().cron_expression(), move |_uuid, _l| {
        let cloned_state = state.clone();
        let cloned_job = job_instance.clone();
        Box::pin(async move {
            cloned_job.run(cloned_state).await;
        })
    })
    .unwrap();

    scheduler.lock().await.add(job).await.unwrap();
}

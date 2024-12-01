use crate::{hackmd_process::fetch_notes_job, state::AppStateV2};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn initialize_scheduler(state: AppStateV2) -> Arc<Mutex<JobScheduler>> {
    let scheduler = Arc::new(Mutex::new(JobScheduler::new().await.unwrap()));

    let job_state = state.clone();
    let scheduler_clone = scheduler.clone();

    tokio::spawn(async move {
        let job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let job_state = job_state.clone();
            Box::pin(async move {
                let _ = fetch_notes_job(job_state).await;
            })
        })
        .unwrap();

        scheduler_clone.lock().await.add(job).await.unwrap();
        scheduler_clone.lock().await.start().await.unwrap();
    });

    scheduler
}

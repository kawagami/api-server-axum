use crate::{jobs::hackmd::FetchNotesJob, state::AppStateV2, structs::jobs::AppJob};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn initialize_scheduler(state: AppStateV2) -> Arc<Mutex<JobScheduler>> {
    let scheduler = Arc::new(Mutex::new(JobScheduler::new().await.unwrap()));

    let job_state = state.clone();
    let scheduler_clone = scheduler.clone();

    tokio::spawn(async move {
        let notes_job = FetchNotesJob;

        let job = Job::new_async(notes_job.clone().cron_expression(), move |_uuid, _l| {
            let job_state = job_state.clone();
            let notes_job = notes_job.clone();
            Box::pin(async move {
                notes_job.run(job_state).await;
            })
        })
        .unwrap();

        scheduler_clone.lock().await.add(job).await.unwrap();
        scheduler_clone.lock().await.start().await.unwrap();
    });

    scheduler
}

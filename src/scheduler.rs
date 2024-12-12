use crate::{
    jobs::{example::ExampleJob, hackmd::FetchNotesJob},
    state::AppStateV2,
    structs::jobs::AppJob,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn initialize_scheduler(state: AppStateV2) -> Arc<Mutex<JobScheduler>> {
    let scheduler = Arc::new(Mutex::new(JobScheduler::new().await.unwrap()));

    let cloned_state = state.clone();
    let cloned_scheduler = scheduler.clone();

    tokio::spawn(async move {
        add_job_to_scheduler(
            cloned_scheduler.clone(),
            cloned_state.clone(),
            FetchNotesJob,
        )
        .await;
        add_job_to_scheduler(cloned_scheduler.clone(), cloned_state.clone(), ExampleJob).await;

        cloned_scheduler.lock().await.start().await.unwrap();
    });

    scheduler
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

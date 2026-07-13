use crate::{repositories::system_metrics as metrics_repo, services::system_metrics, state::AppState};

/// 每分鐘採集一筆 VPS 系統指標寫入 system_metrics。
pub async fn run(state: AppState) {
    let sample = match system_metrics::collect().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("collect_system_metrics: collect failed: {e}");
            return;
        }
    };

    if let Err(e) = metrics_repo::insert(state.get_pool(), &sample).await {
        tracing::error!("collect_system_metrics: insert failed: {e}");
    }
}

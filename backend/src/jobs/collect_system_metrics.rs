use crate::{repositories::system_metrics as metrics_repo, services::system_metrics, state::AppState};

/// 每分鐘採集一筆 VPS 系統指標寫入 system_metrics。
///
/// CPU 是本輪與上一輪 /proc/stat 累計值相減得到的**整分鐘平均**,所以新基準無論這輪有沒有
/// 落地都要存回 state —— 少存一次的話下一輪也沒有基準,會一路空轉下去。
pub async fn run(state: AppState) {
    let (sample, now) = match system_metrics::collect(state.cpu_times()) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("collect_system_metrics: collect failed: {e}");
            return;
        }
    };
    state.set_cpu_times(now);

    // 行程剛起來的第一輪沒有基準可減(或累計值倒退),CPU 算不出來就整筆跳過
    let Some(sample) = sample else {
        tracing::debug!("collect_system_metrics: 尚無 CPU 基準,本輪不落地");
        return;
    };

    if let Err(e) = metrics_repo::insert(state.get_pool(), &sample).await {
        tracing::error!("collect_system_metrics: insert failed: {e}");
    }
}

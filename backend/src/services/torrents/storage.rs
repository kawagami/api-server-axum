use crate::{
    errors::{AppError, SystemError},
    repositories::torrents as torrents_repo,
    state::AppState,
};
use super::manager::{DEFAULT_MAX_TOTAL_SIZE_GB, setting};

/// 儲存空間概況：TORRENT_PATH 所在檔案系統的實際剩餘 + torrent 配額用量
pub async fn storage_stats(state: &AppState) -> Result<serde_json::Value, AppError> {
    let manager = state.get_torrents();
    let (disk_total, disk_available) = disk_space(manager.base_path())
        .map_err(|e| SystemError::Internal(format!("statvfs failed: {e}")))?;

    let used = torrents_repo::total_size_sum(state.get_pool()).await?;
    let max_bytes = setting(state, "torrent_max_total_size_gb", DEFAULT_MAX_TOTAL_SIZE_GB)
        .saturating_mul(1024 * 1024 * 1024);

    Ok(serde_json::json!({
        "disk": {
            "total_bytes": disk_total,
            "available_bytes": disk_available,
        },
        "torrent": {
            "used_bytes": used,
            "max_bytes": max_bytes,
        },
    }))
}

/// statvfs 查路徑所在檔案系統的 (總容量, 非 root 可用容量)，單位 bytes
fn disk_space(path: &std::path::Path) -> std::io::Result<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let total = stat.f_blocks as u64 * stat.f_frsize as u64;
    let available = stat.f_bavail as u64 * stat.f_frsize as u64;
    Ok((total, available))
}

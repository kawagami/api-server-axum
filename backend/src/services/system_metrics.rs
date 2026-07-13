//! VPS 系統指標採集:直接讀 /proc + libc::statvfs,不引外部 crate(container 內 /proc 由 kernel 掛載)。
//! 容器未設 cgroup 上限時,這些值即等同整台 VPS。

use crate::repositories::system_metrics::MetricSample;
use std::io;

const DISK_PATH: &str = "/";

/// 採集一筆快照。CPU% 需兩次 /proc/stat 取樣,故為 async(中間 sleep)。
pub async fn collect() -> io::Result<MetricSample> {
    let cpu_pct = sample_cpu().await?;
    let (mem_used_mb, mem_total_mb) = read_mem()?;
    let (disk_used_mb, disk_total_mb) = read_disk(DISK_PATH)?;
    let (load1, load5, load15) = read_loadavg()?;

    Ok(MetricSample {
        cpu_pct,
        mem_used_mb,
        mem_total_mb,
        disk_used_mb,
        disk_total_mb,
        load1,
        load5,
        load15,
    })
}

/// 兩次讀 /proc/stat 首行(間隔 500ms),算這段期間的整機 CPU 使用率。
async fn sample_cpu() -> io::Result<f32> {
    let (idle1, total1) = read_cpu_times()?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let (idle2, total2) = read_cpu_times()?;

    let d_total = total2.saturating_sub(total1);
    let d_idle = idle2.saturating_sub(idle1);
    if d_total == 0 {
        return Ok(0.0);
    }
    let busy = d_total.saturating_sub(d_idle) as f32;
    Ok((busy / d_total as f32 * 100.0).clamp(0.0, 100.0))
}

/// 回傳 (idle_ticks, total_ticks)。idle = idle + iowait。
fn read_cpu_times() -> io::Result<(u64, u64)> {
    let stat = std::fs::read_to_string("/proc/stat")?;
    let line = stat
        .lines()
        .next()
        .filter(|l| l.starts_with("cpu "))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no cpu line in /proc/stat"))?;

    let nums: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();
    // user nice system idle iowait irq softirq steal ...
    if nums.len() < 5 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "malformed /proc/stat"));
    }
    let idle = nums[3] + nums[4]; // idle + iowait
    let total: u64 = nums.iter().sum();
    Ok((idle, total))
}

/// 讀 /proc/meminfo,回傳 (used_mb, total_mb)。used = total - available。
fn read_mem() -> io::Result<(i32, i32)> {
    let info = std::fs::read_to_string("/proc/meminfo")?;
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;
    for line in info.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = parse_kb(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            avail_kb = parse_kb(rest);
        }
    }
    if total_kb == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "no MemTotal"));
    }
    let total_mb = (total_kb / 1024) as i32;
    let used_mb = (total_kb.saturating_sub(avail_kb) / 1024) as i32;
    Ok((used_mb, total_mb))
}

fn parse_kb(s: &str) -> u64 {
    s.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0)
}

/// 讀 /proc/loadavg,回傳 (1m, 5m, 15m)。
fn read_loadavg() -> io::Result<(f32, f32, f32)> {
    let s = std::fs::read_to_string("/proc/loadavg")?;
    let mut it = s.split_whitespace();
    let l1 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let l5 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let l15 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    Ok((l1, l5, l15))
}

/// statvfs 查磁碟,回傳 (used_mb, total_mb)。used = total - free。
fn read_disk(path: &str) -> io::Result<(i32, i32)> {
    let c_path = std::ffi::CString::new(path)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let frsize = stat.f_frsize as u64;
    let total = stat.f_blocks as u64 * frsize;
    let free = stat.f_bfree as u64 * frsize;
    let used = total.saturating_sub(free);
    let mb = 1024 * 1024;
    Ok(((used / mb) as i32, (total / mb) as i32))
}

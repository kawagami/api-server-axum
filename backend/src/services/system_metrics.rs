//! VPS 系統指標採集:直接讀 /proc + libc::statvfs,不引外部 crate(container 內 /proc 由 kernel 掛載)。
//! 容器未設 cgroup 上限時,這些值即等同整台 VPS。

use crate::repositories::system_metrics::MetricSample;
use std::io;

const DISK_PATH: &str = "/";

/// /proc/stat 首行的累計 tick 數。單獨一筆沒有意義,要跟前一筆相減才得到區間使用率。
#[derive(Clone, Copy, Debug)]
pub struct CpuTimes {
    /// user + nice + system + irq + softirq —— guest 真的在算東西的時間
    pub busy: u64,
    /// hypervisor 抽走 vCPU 的時間。**不算進 busy**:那不是本機在忙
    pub steal: u64,
    /// busy + steal + idle + iowait
    pub total: u64,
}

/// 採集一筆快照。
///
/// CPU 用「與上一次採樣的累計 tick 差」算**整個採樣間隔的平均**,所以 `prev` 是必要輸入,
/// 而且無論成敗都要把回傳的 [`CpuTimes`] 存回去當下次的基準。`prev = None`(開機後第一次)
/// 沒有基準可減,回 `None` 表示這輪不落地。
///
/// ⚠️ **不要改回「當場 sleep 幾百毫秒取兩次」的寫法**(2026-08-08 前是 sleep 500ms):
/// 那個窗口的起點固定落在 cron 秒 0,正好是同秒觸發的另兩個每分鐘 job 在打 DB 的瞬間,
/// 於是每分鐘都系統性地只量到最忙的那 500ms —— 實測 vmstat 開機以來平均 busy 3%、
/// load1 中位數 0.00,這頁卻長期顯示 34%,差 10 倍。跨間隔相減沒有窗口可挑,也不必 sleep。
pub fn collect(prev: Option<CpuTimes>) -> io::Result<(Option<MetricSample>, CpuTimes)> {
    let now = read_cpu_times()?;
    let Some((cpu_pct, cpu_steal_pct)) = prev.and_then(|p| cpu_delta(p, now)) else {
        return Ok((None, now));
    };

    let (mem_used_mb, mem_total_mb) = read_mem()?;
    let (disk_used_mb, disk_total_mb) = read_disk(DISK_PATH)?;
    let (load1, load5, load15) = read_loadavg()?;
    // backend 行程自身 RSS:與整機 mem 分開,才能辨別「是我 Rust 在漲」還是 PG/Redis/前端。
    let backend_rss_mb = read_self_rss().unwrap_or(0);

    let sample = MetricSample {
        cpu_pct,
        cpu_steal_pct,
        mem_used_mb,
        mem_total_mb,
        disk_used_mb,
        disk_total_mb,
        load1,
        load5,
        load15,
        backend_rss_mb,
    };
    Ok((Some(sample), now))
}

/// 兩筆累計值相減得 (busy%, steal%)。
/// `None` = 這對值算不出東西:計數器倒退(主機重開)或間隔內完全沒有 tick。
fn cpu_delta(prev: CpuTimes, now: CpuTimes) -> Option<(f32, f32)> {
    let d_total = now.total.checked_sub(prev.total)?;
    if d_total == 0 {
        return None;
    }
    let pct = |d: u64| (d as f32 / d_total as f32 * 100.0).clamp(0.0, 100.0);
    Some((
        pct(now.busy.saturating_sub(prev.busy)),
        pct(now.steal.saturating_sub(prev.steal)),
    ))
}

/// 讀 /proc/stat 首行的累計 tick。
fn read_cpu_times() -> io::Result<CpuTimes> {
    parse_cpu_times(&std::fs::read_to_string("/proc/stat")?)
}

fn parse_cpu_times(stat: &str) -> io::Result<CpuTimes> {
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
    // user nice system idle iowait irq softirq steal guest guest_nice
    if nums.len() < 5 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "malformed /proc/stat"));
    }
    let at = |i: usize| nums.get(i).copied().unwrap_or(0);

    let idle = at(3) + at(4); // idle + iowait:兩者都代表 CPU 沒在算東西
    let steal = at(7); // 2.6.11 以前的 kernel 沒這欄
    // guest / guest_nice 刻意不加進 total:kernel 已把 guest 併入 user、guest_nice 併入 nice,
    // 整行加總會重複計算、讓分母虛胖(只有跑 KVM guest 的 host 非零,這裡恆為 0,但別留著等踩)。
    let total: u64 = nums.iter().take(8).sum();
    Ok(CpuTimes {
        busy: total.saturating_sub(idle).saturating_sub(steal),
        steal,
        total,
    })
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

/// 讀 /proc/self/status 的 VmRSS,回傳本行程常駐記憶體(MB)。
/// 容器內 /proc/self 恆為此 backend 行程,不含 PG/Redis/前端。
fn read_self_rss() -> io::Result<i32> {
    let status = std::fs::read_to_string("/proc/self/status")?;
    let rss_kb = status
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:").map(parse_kb))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no VmRSS in /proc/self/status"))?;
    Ok((rss_kb / 1024) as i32)
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

#[cfg(test)]
mod tests {
    use super::{cpu_delta, parse_cpu_times};

    // user nice system idle iowait irq softirq steal guest guest_nice
    fn stat(fields: [u64; 10]) -> String {
        let nums = fields.map(|v| v.to_string()).join(" ");
        format!("cpu  {nums}\ncpu0 1 2 3 4 5 6 7 8 9 10\nintr 0\n")
    }

    #[test]
    fn steal_is_not_counted_as_busy() {
        // 這是 2026-08-08 之前那版把 33% steal 顯示成「CPU 使用率」的成因
        let t = parse_cpu_times(&stat([10, 0, 5, 100, 0, 1, 4, 30, 0, 0])).unwrap();
        assert_eq!(t.busy, 20); // 10+0+5+1+4
        assert_eq!(t.steal, 30);
        assert_eq!(t.total, 150);
    }

    #[test]
    fn guest_fields_do_not_inflate_total() {
        // kernel 已把 guest 併入 user、guest_nice 併入 nice,整行加總會重複計算
        let a = parse_cpu_times(&stat([10, 0, 5, 100, 0, 0, 0, 0, 0, 0])).unwrap();
        let b = parse_cpu_times(&stat([10, 0, 5, 100, 0, 0, 0, 0, 7, 3])).unwrap();
        assert_eq!(a.total, b.total);
    }

    #[test]
    fn missing_steal_field_on_old_kernels() {
        let t = parse_cpu_times("cpu  10 0 5 100 0\n").unwrap();
        assert_eq!(t.busy, 15);
        assert_eq!(t.steal, 0);
        assert_eq!(t.total, 115);
    }

    #[test]
    fn delta_is_the_interval_average() {
        // 6000 ticks(1 核 60 秒)裡忙 180 → 3%,而非「挑最忙的 500ms」的 34%
        let prev = parse_cpu_times(&stat([100, 0, 50, 1000, 0, 0, 0, 0, 0, 0])).unwrap();
        let now = parse_cpu_times(&stat([220, 0, 110, 6820, 0, 0, 0, 0, 0, 0])).unwrap();
        let (busy, steal) = cpu_delta(prev, now).unwrap();
        assert!((busy - 3.0).abs() < 0.01, "busy={busy}");
        assert_eq!(steal, 0.0);
    }

    #[test]
    fn counter_reset_yields_no_sample() {
        // 主機重開後累計值倒退:寧可漏一筆也不要算出天文數字
        let prev = parse_cpu_times(&stat([100, 0, 50, 1000, 0, 0, 0, 0, 0, 0])).unwrap();
        let now = parse_cpu_times(&stat([1, 0, 1, 10, 0, 0, 0, 0, 0, 0])).unwrap();
        assert!(cpu_delta(prev, now).is_none());
        assert!(cpu_delta(prev, prev).is_none()); // 間隔內零 tick
    }
}

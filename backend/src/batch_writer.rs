//! 觀測資料的批次寫入迴圈 —— `logs` 與 `admin_audit_logs` 共用這一份。
//!
//! 兩者的需求逐字相同：**請求路徑上不碰 DB**（尖峰時不與真正的查詢搶那 20 條 PG 連線），
//! 改成丟進 channel、由一個背景 task 攢批寫。原本是兩份各自帶著 `BATCH_SIZE` /
//! `FLUSH_INTERVAL_MS` / `CHANNEL_CAPACITY` 三個常數的同構 `select!` 迴圈 ——
//! 六個常數、兩份迴圈，改一邊忘另一邊不會有任何提示，所以收成這一份。
//!
//! 這裡只管「什麼時候該寫」，寫什麼、寫失敗怎麼辦留給呼叫端的 `flush`：
//! 那兩件事兩邊**真的不同**（log 的失敗不能用 `tracing` 回報，會遞迴回 `on_event`）。

use std::future::Future;
use tokio::sync::mpsc;

/// channel 容量。滿了就丟棄新進的紀錄 —— 觀測資料不該把請求拖慢，也不該把 DB 連線耗光。
/// 丟棄本身必須看得見，各呼叫端自己負責（見 `logging::report_dropped`）。
pub const CHANNEL_CAPACITY: usize = 1000;

/// 累積到這個數量就寫一次
const BATCH_SIZE: usize = 50;

/// 沒累積滿也至少這麼久寫一次
const FLUSH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);

/// 批次寫入迴圈。channel 關閉（正常關機）時把剩下的寫完才結束。
///
/// - `flush` — 把一批資料落地。**空批次不會被呼叫**。
/// - `on_interval` — 每個 flush 間隔呼叫一次，**不論這輪有沒有資料**。存在的理由只有
///   `logging` 的丟棄數匯總：那件事得在「佇列滿到連 buf 都攢不出東西」時照樣發生。
///   沒有這種需求的呼叫端傳 `|| {}`。
pub async fn run<T, F, Fut>(mut rx: mpsc::Receiver<T>, mut flush: F, mut on_interval: impl FnMut())
where
    F: FnMut(Vec<T>) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut buf: Vec<T> = Vec::with_capacity(BATCH_SIZE);
    let mut interval = tokio::time::interval(FLUSH_INTERVAL);

    loop {
        tokio::select! {
            entry = rx.recv() => {
                match entry {
                    Some(e) => {
                        buf.push(e);
                        if buf.len() >= BATCH_SIZE {
                            flush(take(&mut buf)).await;
                        }
                    }
                    None => {
                        if !buf.is_empty() {
                            flush(take(&mut buf)).await;
                        }
                        return;
                    }
                }
            }
            _ = interval.tick() => {
                if !buf.is_empty() {
                    flush(take(&mut buf)).await;
                }
                on_interval();
            }
        }
    }
}

/// 取走整批、原地換一個保有容量的空 buffer（`mem::take` 會退回容量 0，每輪重新配置）
fn take<T>(buf: &mut Vec<T>) -> Vec<T> {
    std::mem::replace(buf, Vec::with_capacity(BATCH_SIZE))
}

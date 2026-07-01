// setInterval(1000) 會漂移，顯示秒數偶爾跳兩秒；
// 改用每次 tick 重新對齊秒邊界的 setTimeout（+20ms 緩衝避免提早觸發）。
// anchor 為對齊基準的 epoch ms（倒數計時傳 targetTime），預設 0 = 對齊系統時鐘整秒。
export function startSecondTick(cb: () => void, anchor = 0): () => void {
    let timer: ReturnType<typeof setTimeout>;
    function schedule() {
        const ms = ((anchor - Date.now()) % 1000 + 1000) % 1000 || 1000;
        timer = setTimeout(() => {
            cb();
            schedule();
        }, ms + 20);
    }
    cb();
    schedule();
    return () => clearTimeout(timer);
}

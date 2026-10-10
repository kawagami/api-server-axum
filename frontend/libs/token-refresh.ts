let refreshTimer: ReturnType<typeof setInterval> | null = null;

// session 是 httpOnly cookie，同源 fetch 自動帶上，client 端不經手 token
async function doRefresh(): Promise<void> {
    try {
        const res = await fetch('/api/auth/refresh', { method: 'POST' });
        if (!res.ok) {
            stopTokenRefresh();
            // eslint-disable-next-line @next/next/no-location-assign-relative-destination -- session 已失效，刻意整頁重載清掉 client 端狀態
            window.location.href = '/admin/login';
        }
    } catch {
        // 網路暫時異常：保留 timer，下一輪再試
    }
}

export function startTokenRefresh(): void {
    stopTokenRefresh();
    refreshTimer = setInterval(doRefresh, 50 * 60 * 1000);
}

export function stopTokenRefresh(): void {
    if (refreshTimer !== null) {
        clearInterval(refreshTimer);
        refreshTimer = null;
    }
}

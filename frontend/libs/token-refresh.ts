let refreshTimer: ReturnType<typeof setInterval> | null = null;

async function doRefresh(): Promise<void> {
    const token = localStorage.getItem('token');
    if (!token) {
        stopTokenRefresh();
        return;
    }

    const res = await fetch('/api/auth/refresh', {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
    });

    if (res.ok) {
        const { token: newToken } = await res.json();
        localStorage.setItem('token', newToken);
    } else {
        stopTokenRefresh();
        localStorage.removeItem('token');
        window.location.href = '/admin/login';
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

export function restartTokenRefresh(): void {
    if (localStorage.getItem('token')) {
        startTokenRefresh();
    }
}

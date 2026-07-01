"use client";

import { createContext, useContext, useEffect, useMemo, useRef, useState } from 'react';

// 下行訊息外殼。遊戲框架信封含 `game` 欄（chess/gomoku/banqi）；
// 非遊戲訊息（user_joined 等）無 game。listener 第二參數拿到整則 msg 供過濾。
export interface WsMessage { type: string; game?: string; data?: unknown }
type Listener = (data: unknown, msg: WsMessage) => void;
// 重連成功後（非首次連上）會呼叫的 init 回呼，讓畫面依當下狀態重送初始訊息
type ReconnectHandler = () => void;

interface WsContextValue {
    subscribe: (type: string, fn: Listener) => void;
    unsubscribe: (type: string, fn: Listener) => void;
    // game：遊戲框架信封需帶（如 'chess'）；一般 WS 訊息省略
    send: (type: string, data?: unknown, game?: string) => void;
    // 註冊「重連後重送初始訊息」的回呼。server 重啟會遺失全部狀態（大廳訂閱/對局/連線記錄），
    // 重連成功後本回呼會被呼叫，consumer 應依當下畫面重送 join_lobby 等初始訊息。
    // 只在「重連」時觸發，首次連線不呼叫（首次 init 仍由 consumer mount 時自行送）。
    // 回傳取消註冊函式。
    onReconnect: (fn: ReconnectHandler) => () => void;
}

const WsContext = createContext<WsContextValue | null>(null);

const RECONNECT_BASE_MS = 1000;
const RECONNECT_MAX_MS = 30000;
// 連續重連失敗超過此次數（約 75s）才顯示「請重新整理」提示，背景仍持續嘗試
const RECONNECT_FAIL_THRESHOLD = 5;

export function WsProvider({ children, jwt, wsUrl }: { children: React.ReactNode; jwt: string | null; wsUrl: string }) {
    const listenersRef = useRef<Map<string, Set<Listener>>>(new Map());
    const reconnectHandlersRef = useRef<Set<ReconnectHandler>>(new Set());
    const wsRef = useRef<WebSocket | null>(null);
    // open 前送的訊息暫存，onopen 時 flush（給 mount 即送的 join_queue 用）
    const pendingRef = useRef<string[]>([]);
    // 連線是否曾經成功過：用來區分「首次連上」與「重連連上」
    const hasConnectedRef = useRef(false);
    // 長時間連不上時顯示提示（背景仍重試）
    const [lostConnection, setLostConnection] = useState(false);

    useEffect(() => {
        // WS_URL 未設定時直接不建立連線（例如本地後端沒開）
        if (!wsUrl) return;

        let destroyed = false;
        let attempt = 0;
        let timeoutId: ReturnType<typeof setTimeout>;

        const url = jwt ? `${wsUrl}/ws?jwt=${jwt}` : `${wsUrl}/ws`;

        const scheduleReconnect = () => {
            if (destroyed) return;
            // 指數退避 + jitter（避免多 client 同時重連打爆 server）
            const capped = Math.min(RECONNECT_BASE_MS * 2 ** attempt, RECONNECT_MAX_MS);
            const delay = capped * (0.5 + Math.random() * 0.5);
            attempt++;
            if (attempt > RECONNECT_FAIL_THRESHOLD) setLostConnection(true);
            timeoutId = setTimeout(connect, delay);
        };

        const connect = () => {
            const ws = new WebSocket(url);
            wsRef.current = ws;

            ws.onmessage = (event: MessageEvent) => {
                try {
                    const msg = JSON.parse(event.data as string) as WsMessage;
                    listenersRef.current.get(msg.type)?.forEach(fn => fn(msg.data, msg));
                } catch {
                    // ignore non-JSON frames
                }
            };

            ws.onopen = () => {
                attempt = 0;
                setLostConnection(false);
                const pending = pendingRef.current;
                pendingRef.current = [];
                pending.forEach(frame => ws.send(frame));
                // 重連（非首次）：server 已遺失全部狀態，呼叫 consumer 重送初始訊息恢復畫面
                if (hasConnectedRef.current) {
                    reconnectHandlersRef.current.forEach(fn => {
                        try { fn(); } catch { /* ignore handler error */ }
                    });
                }
                hasConnectedRef.current = true;
            };

            // onclose / onerror 幾乎都代表 server 重啟或網路中斷，皆觸發重連。
            // 先清掉 handler 再排程，避免舊 socket 殘留事件造成多條連線。
            const handleDown = () => {
                ws.onopen = ws.onmessage = ws.onerror = ws.onclose = null;
                if (wsRef.current === ws) wsRef.current = null;
                scheduleReconnect();
            };
            ws.onclose = handleDown;
            ws.onerror = () => {
                // onerror 後瀏覽器通常接著觸發 onclose；主動 close 以統一走 handleDown
                try { ws.close(); } catch { /* already closing */ }
            };
        };

        connect();

        return () => {
            destroyed = true;
            clearTimeout(timeoutId);
            const ws = wsRef.current;
            if (ws) {
                // 清掉 handler 再關，避免 cleanup 觸發的 close 又排程重連
                ws.onopen = ws.onmessage = ws.onerror = ws.onclose = null;
                ws.close();
            }
            wsRef.current = null;
            hasConnectedRef.current = false;
        };
    }, [jwt, wsUrl]);

    const value = useMemo<WsContextValue>(() => ({
        subscribe: (type, fn) => {
            const map = listenersRef.current;
            if (!map.has(type)) map.set(type, new Set());
            map.get(type)!.add(fn);
        },
        unsubscribe: (type, fn) => {
            listenersRef.current.get(type)?.delete(fn);
        },
        send: (type, data, game) => {
            const envelope: WsMessage = { type };
            if (game !== undefined) envelope.game = game;
            if (data !== undefined) envelope.data = data;
            const frame = JSON.stringify(envelope);
            const ws = wsRef.current;
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(frame);
            } else {
                // 連線未開：暫存，onopen flush
                pendingRef.current.push(frame);
            }
        },
        onReconnect: (fn) => {
            reconnectHandlersRef.current.add(fn);
            return () => { reconnectHandlersRef.current.delete(fn); };
        },
    }), []);

    return (
        <WsContext.Provider value={value}>
            {children}
            {lostConnection && <WsLostBanner />}
        </WsContext.Provider>
    );
}

// 連線長時間中斷提示（背景仍持續重連，恢復後自動消失）。
// WsProvider 掛在 root layout（NextIntlClientProvider 之外），無法用 useTranslations，
// 文案固定走預設語系 zh-TW。
function WsLostBanner() {
    return (
        <div
            role="status"
            className="fixed bottom-4 left-1/2 z-50 -translate-x-1/2 rounded-lg bg-red-600 px-4 py-2 text-sm text-white shadow-lg"
        >
            連線中斷，重新連線中…若持續未恢復請重新整理頁面
        </div>
    );
}

export function useWsContext() {
    const ctx = useContext(WsContext);
    if (!ctx) throw new Error('useWsContext must be used within WsProvider');
    return ctx;
}

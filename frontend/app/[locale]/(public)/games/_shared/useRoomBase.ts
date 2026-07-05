"use client";

import { useEffect, useRef, useState } from 'react';
import { useWsContext, type WsMessage } from '@/libs/ws-context';

// 三套 room hook（useGameRoom / useAvalonRoom / useFarmRoom）的共用底座：
// 訂閱以 game 分流、error→notice 映射、進頁 join_lobby、切頁 leave、重連重進大廳。
// 狀態機本體（2 人對戰 vs N 人房）差異大，仍由各 hook 自持，這裡只收機械邏輯。
export interface RoomBaseOptions {
    game: string;                                       // 信封 game 欄，訂閱分流用
    knownErrors: Set<string>;                           // error.reason 白名單 → notice `err_${reason}`
    genericErrorKey?: string;                           // 非白名單 reason 的 i18n key（預設 err_generic）
    handlers: Record<string, (data: unknown) => void>;  // 各遊戲訊息 handler（error 由底座處理）；每次 render 取最新，可直接閉包 state，不需 memo
    onReconnectReset: () => void;                       // 重連後（底座已重送 join_lobby）各遊戲清房內狀態、回 connecting
    leaveOnUnmount: () => void;                         // 切離頁面時依當下 phase 送 leave/resign（共用 socket 不會因切頁斷線）
}

export interface RoomBase {
    notice: string | null;                      // i18n key
    setNotice: (notice: string | null) => void;
}

export function useRoomBase(opts: RoomBaseOptions): RoomBase {
    const { subscribe, unsubscribe, send, onReconnect } = useWsContext();
    const { game } = opts;

    const [notice, setNotice] = useState<string | null>(null);

    // options 最新值（handlers/callbacks 閉包當下 state，避免反覆 sub/unsub）
    const optsRef = useRef(opts);
    useEffect(() => { optsRef.current = opts; });

    useEffect(() => {
        // game 分流：只處理本遊戲訊息；error 統一映射 notice
        const types = [...Object.keys(optsRef.current.handlers), 'error'];
        const entries = types.map(type => {
            const guarded = (data: unknown, msg: WsMessage) => {
                if (msg.game !== game) return;
                if (type === 'error') {
                    const { knownErrors, genericErrorKey = 'err_generic' } = optsRef.current;
                    const r = (data as { reason: string }).reason;
                    setNotice(knownErrors.has(r) ? `err_${r}` : genericErrorKey);
                } else {
                    optsRef.current.handlers[type]?.(data);
                }
            };
            return [type, guarded] as const;
        });
        entries.forEach(([type, fn]) => subscribe(type, fn));
        return () => entries.forEach(([type, fn]) => unsubscribe(type, fn));
    }, [subscribe, unsubscribe, game]);

    // 進頁進大廳（一次）
    const startedRef = useRef(false);
    useEffect(() => {
        if (startedRef.current) return;
        startedRef.current = true;
        send('join_lobby', undefined, game);
    }, [send, game]);

    // 切離頁面主動退出：送什麼由各遊戲依當下 phase 決定
    useEffect(() => () => optsRef.current.leaveOnUnmount(), []);

    // 重連後 server 已遺失大廳訂閱與房間/對局狀態：重送 join_lobby 取回大廳，其餘重設交給各遊戲
    useEffect(() => onReconnect(() => {
        send('join_lobby', undefined, game);
        optsRef.current.onReconnectReset();
    }), [onReconnect, send, game]);

    return { notice, setNotice };
}

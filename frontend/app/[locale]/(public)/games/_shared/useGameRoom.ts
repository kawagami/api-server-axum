"use client";

import { useCallback, useState } from 'react';
import { useWsContext } from '@/libs/ws-context';
import { useRoomBase } from './useRoomBase';
import { sound } from './sound';
import type {
    GameId, WireTable, TableListData, TableCreatedData, QueuedData,
    MatchFoundData, MoveMadeBase, GameOverData,
} from './wire';

export type RoomPhase = 'connecting' | 'lobby' | 'queued' | 'hosting' | 'playing' | 'over';

// 大廳/桌位/防呆 error.reason（snake），三遊戲共用
const KNOWN_ERR = new Set([
    'already_committed', 'bad_table_id', 'table_not_found',
    'table_full', 'cannot_join_self', 'not_in_game', 'game_ended',
]);

export interface GameRoomCallbacks {
    onMatchFound: (myColor: string) => void; // 各遊戲重設盤面
    onMoveMade: (data: unknown) => void;      // 各遊戲套用走步（data 含 turn/clock + 自有欄位）
    onCheck?: (data: unknown) => void;        // 象棋專用
}

export interface UseGameRoom {
    phase: RoomPhase;
    tables: WireTable[];
    queuePos: number;
    hostedTableId: number | null;
    notice: string | null;     // i18n key（GameLobby namespace）
    myColor: string;
    turn: string;
    clock: Record<string, number>;
    result: GameOverData | null;
    pending: boolean;
    shake: boolean;
    actions: {
        quickMatch: () => void;
        createTable: (name: string) => void;
        joinTable: (id: number) => void;
        leaveQueue: () => void;
        cancelHost: () => void;
        sendMove: (data: unknown) => void;
        resign: () => void;
        backToLobby: () => void;
    };
}

// game：遊戲 id；sides：[先手, 後手] 標籤（決定初始 turn 與 clock 鍵）
export function useGameRoom(game: GameId, sides: readonly [string, string], cb: GameRoomCallbacks): UseGameRoom {
    const { send } = useWsContext();

    const [phase, setPhase] = useState<RoomPhase>('connecting');
    const [tables, setTables] = useState<WireTable[]>([]);
    const [queuePos, setQueuePos] = useState(0);
    const [hostedTableId, setHostedTableId] = useState<number | null>(null);
    const [myColor, setMyColor] = useState<string>(sides[0]);
    const [turn, setTurn] = useState<string>(sides[0]);
    const [clock, setClock] = useState<Record<string, number>>({ [sides[0]]: 300000, [sides[1]]: 300000 });
    const [result, setResult] = useState<GameOverData | null>(null);
    const [pending, setPending] = useState(false);
    const [shake, setShake] = useState(false);

    const { notice, setNotice } = useRoomBase({
        game,
        knownErrors: KNOWN_ERR,
        genericErrorKey: 'errGeneric',
        handlers: {
            table_list: d => {
                setTables((d as TableListData).tables);
                setPhase(p => (p === 'connecting' ? 'lobby' : p));
            },
            lobby_update: d => setTables((d as TableListData).tables),
            table_created: d => {
                setHostedTableId((d as TableCreatedData).table_id);
                setNotice(null);
                setPhase('hosting');
            },
            queued: d => {
                setQueuePos((d as QueuedData).position);
                setNotice(null);
                setPhase('queued');
            },
            match_found: d => {
                const { color, clock_ms } = d as MatchFoundData;
                setMyColor(color);
                setTurn(sides[0]);
                setClock({ [sides[0]]: clock_ms, [sides[1]]: clock_ms });
                setResult(null);
                setPending(false);
                setHostedTableId(null);
                setNotice(null);
                cb.onMatchFound(color);
                setPhase('playing');
            },
            move_made: d => {
                const m = d as MoveMadeBase;
                setTurn(m.turn);
                setClock(m.clock);
                setPending(false);
                cb.onMoveMade(d);
            },
            check: d => {
                sound.check();
                cb.onCheck?.(d);
            },
            game_over: d => {
                setResult(d as GameOverData);
                setPending(false);
                setPhase('over');
                sound.gameOver();
            },
            illegal_move: () => {
                setShake(true);
                setPending(false);
                setTimeout(() => setShake(false), 400);
            },
        },
        // 佇列/桌位/對局在 server 重啟後已不存在，故非大廳一律回 connecting，等 table_list 重建畫面
        onReconnectReset: () => setPhase(p => (p === 'lobby' ? p : 'connecting')),
        // 切離頁面依當下 phase 送對應退出指令
        leaveOnUnmount: () => {
            if (phase === 'queued') send('leave_queue', undefined, game);
            else if (phase === 'hosting') send('leave_table', undefined, game);
            else if (phase === 'playing') send('resign', undefined, game);
        },
    });

    const actions = {
        quickMatch: useCallback(() => { setNotice(null); send('join_queue', undefined, game); }, [send, game, setNotice]),
        createTable: useCallback((name: string) => { setNotice(null); send('create_table', name ? { name } : {}, game); }, [send, game, setNotice]),
        joinTable: useCallback((id: number) => { setNotice(null); send('join_table', { table_id: id }, game); }, [send, game, setNotice]),
        leaveQueue: useCallback(() => { send('leave_queue', undefined, game); setPhase('lobby'); }, [send, game]),
        cancelHost: useCallback(() => { send('leave_table', undefined, game); setHostedTableId(null); setPhase('lobby'); }, [send, game]),
        sendMove: useCallback((data: unknown) => { setPending(true); send('move', data, game); }, [send, game]),
        resign: useCallback(() => send('resign', undefined, game), [send, game]),
        backToLobby: useCallback(() => { setNotice(null); setResult(null); send('join_lobby', undefined, game); setPhase('connecting'); }, [send, game, setNotice]),
    };

    return { phase, tables, queuePos, hostedTableId, notice, myColor, turn, clock, result, pending, shake, actions };
}

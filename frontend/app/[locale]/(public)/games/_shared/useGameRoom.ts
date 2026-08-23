"use client";

import { useCallback, useEffect, useRef, useState } from 'react';
import { useWsContext } from '@/libs/ws-context';
import { useRoomBase } from './useRoomBase';
import { sound } from './sound';
import type {
    GameId, WireTable, TableListData, TableCreatedData, QueuedData,
    MatchFoundData, MoveMadeBase, GameOverData, HintsData,
} from './wire';

export type RoomPhase = 'connecting' | 'lobby' | 'queued' | 'hosting' | 'playing' | 'over';

// 大廳/桌位/防呆/連線防護 error.reason（snake），全對戰遊戲共用
const KNOWN_ERR = new Set([
    'already_committed', 'bad_table_id', 'table_not_found',
    'table_full', 'cannot_join_self', 'not_in_game', 'game_ended',
    // 資源上限與連線防護（後端 common/service.rs MAX_TABLES、services/ws.rs 令牌桶）
    'too_many_tables', 'rate_limited',
    // instance 功能開關關掉 games 時，dispatch_game 的回覆
    'feature_disabled',
]);

// 走步被拒的 reason 白名單（→ i18n key `illegal_<reason>`）。**同 KNOWN_ERR 的理由**：
// 沒有白名單的話，引擎新增一個 reason code 就會在畫面上變成一句 next-intl 的缺 key 訊息。
// 來源：common/service.rs（NotYourTurn）＋ 各遊戲 adapter / engine 的錯誤字串。
const ILLEGAL_KEYS = new Set([
    // 框架 / 各 adapter 的解析錯誤
    'NotYourTurn', 'bad_coord', 'bad_action', 'bad_promo', 'illegal', 'no_piece', 'wrong_piece',
    // 象棋
    'NoPiece', 'WrongPiece', 'CaptureOwn', 'BadMove', 'BlockedPath', 'BadHorseLeg',
    'BadElephantEye', 'CrossRiver', 'OutOfPalace', 'FlyingGeneral', 'LeavesKingInCheck',
    // 五子棋 / 圍棋
    'occupied', 'out_of_bounds', 'ko', 'suicide',
    // 暗棋
    'not_hidden', 'already_up', 'not_revealed', 'empty_from', 'not_your_piece', 'no_color_yet',
    'not_adjacent', 'blocked', 'cannon_needs_screen', 'cannot_capture', 'occupied_friendly',
    'target_hidden',
]);

// 送出 move 後等 server 回覆的上限。沒有這個上限的話：封包掉了（或 server 正好重啟）
// 就會永遠停在 pending=true，盤面鎖死且畫面上沒有任何說明，只能重整頁面。
const MOVE_ACK_TIMEOUT_MS = 6000;
// 錯誤提示（不合法的走步 / 逾時）在畫面上停留多久
const MOVE_ERROR_TTL_MS = 4000;
// 對局結束畫面要顯示的最後幾手
const MOVE_LOG_KEEP = 6;

// 取現在時間。包一層是為了 `react-hooks/purity` —— 下面的呼叫點全在 WS handler（事件回呼）
// 裡，不是 render 期，但那條 lint 只看語法位置，直接寫 Date.now() 會被判成 render 期不純。
function nowMs(): number {
    return Date.now();
}

export interface GameRoomCallbacks {
    onMatchFound: (myColor: string) => void; // 各遊戲重設盤面
    onMoveMade: (data: unknown) => void;      // 各遊戲套用走步（data 含 turn/clock + 自有欄位）
    onCheck?: (data: unknown) => void;        // 象棋 / 西洋棋專用
    /// 一手的簡短記譜（結束畫面顯示最後幾手）。省略則不記。
    formatMove?: (data: unknown) => string;
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
    /// 收到 `clock` 那一刻的本地時間（Date.now()）—— `Clock` 用它算出實際剩餘，
    /// 而不是自己每 250ms 累減（背景分頁會被節流，累減必然失準）
    clockAt: number;
    result: GameOverData | null;
    pending: boolean;
    shake: boolean;
    /// server 給當前輪到方的合法步提示（非權威，只用來畫提示）
    hints: HintsData | null;
    /// 走步被拒 / 等不到回覆的說明（i18n key，GameLobby namespace）；數秒後自動消失
    moveError: string | null;
    moveCount: number;
    moveLog: string[];
    /// 對局長度（game_over 時定格）
    durationMs: number | null;
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
    const [clockAt, setClockAt] = useState<number>(nowMs);
    const [result, setResult] = useState<GameOverData | null>(null);
    const [pending, setPending] = useState(false);
    const [shake, setShake] = useState(false);
    const [hints, setHints] = useState<HintsData | null>(null);
    const [moveError, setMoveError] = useState<string | null>(null);
    const [moveCount, setMoveCount] = useState(0);
    const [moveLog, setMoveLog] = useState<string[]>([]);
    const [durationMs, setDurationMs] = useState<number | null>(null);

    const startedAtRef = useRef<number | null>(null);
    const ackTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const errTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const cbRef = useRef(cb);
    useEffect(() => { cbRef.current = cb; });

    const clearAck = useCallback(() => {
        if (ackTimer.current) {
            clearTimeout(ackTimer.current);
            ackTimer.current = null;
        }
    }, []);

    const showMoveError = useCallback((key: string) => {
        setMoveError(key);
        if (errTimer.current) clearTimeout(errTimer.current);
        errTimer.current = setTimeout(() => setMoveError(null), MOVE_ERROR_TTL_MS);
    }, []);

    // 卸載時把計時器收乾淨（切頁後才觸發的 setState 會是 React 的警告來源）
    useEffect(() => () => {
        if (ackTimer.current) clearTimeout(ackTimer.current);
        if (errTimer.current) clearTimeout(errTimer.current);
    }, []);

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
                setClockAt(nowMs());
                setResult(null);
                setPending(false);
                clearAck();
                setHostedTableId(null);
                setNotice(null);
                setHints(null);
                setMoveError(null);
                setMoveCount(0);
                setMoveLog([]);
                setDurationMs(null);
                startedAtRef.current = nowMs();
                cbRef.current.onMatchFound(color);
                setPhase('playing');
            },
            move_made: d => {
                const m = d as MoveMadeBase;
                setTurn(m.turn);
                setClock(m.clock);
                setClockAt(nowMs());
                setPending(false);
                clearAck();
                setMoveCount(n => n + 1);
                // 提示屬於「上一手之前」的局面，收到新走步就先清掉，等 server 推新的
                setHints(null);
                const note = cbRef.current.formatMove?.(d);
                if (note) setMoveLog(log => [...log, note].slice(-MOVE_LOG_KEEP));
                cbRef.current.onMoveMade(d);
            },
            hints: d => setHints(d as HintsData),
            check: d => {
                sound.check();
                cbRef.current.onCheck?.(d);
            },
            game_over: d => {
                setResult(d as GameOverData);
                setPending(false);
                clearAck();
                setHints(null);
                setDurationMs(startedAtRef.current ? nowMs() - startedAtRef.current : null);
                setPhase('over');
                sound.gameOver();
            },
            illegal_move: d => {
                setShake(true);
                setPending(false);
                clearAck();
                // 只抖不說原因的話，新手完全不知道為什麼不能走（server 早就回了 reason）
                const reason = (d as { reason?: string }).reason;
                showMoveError(reason && ILLEGAL_KEYS.has(reason) ? `illegal_${reason}` : 'illegalGeneric');
                setTimeout(() => setShake(false), 400);
            },
        },
        // 佇列/桌位/對局在 server 重啟後已不存在，故非大廳一律回 connecting，等 table_list 重建畫面
        onReconnectReset: () => {
            setPhase(p => (p === 'lobby' ? p : 'connecting'));
            setHints(null);
            setPending(false);
            clearAck();
        },
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
        sendMove: useCallback((data: unknown) => {
            setPending(true);
            setMoveError(null);
            send('move', data, game);
            clearAck();
            ackTimer.current = setTimeout(() => {
                ackTimer.current = null;
                setPending(false);            // 解鎖盤面，讓人可以再試一次
                showMoveError('moveTimeout');
            }, MOVE_ACK_TIMEOUT_MS);
        }, [send, game, clearAck, showMoveError]),
        resign: useCallback(() => send('resign', undefined, game), [send, game]),
        backToLobby: useCallback(() => { setNotice(null); setResult(null); send('join_lobby', undefined, game); setPhase('connecting'); }, [send, game, setNotice]),
    };

    return {
        phase, tables, queuePos, hostedTableId, notice, myColor, turn, clock, clockAt,
        result, pending, shake, hints, moveError, moveCount, moveLog, durationMs, actions,
    };
}

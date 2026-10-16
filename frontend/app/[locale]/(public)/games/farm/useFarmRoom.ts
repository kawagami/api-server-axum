"use client";

import { useCallback, useRef, useState } from 'react';
import { useWsContext } from '@/libs/ws-context';
import { useRoomBase } from '../_shared/useRoomBase';
import {
    GAME, type RoomSummary, type RoomListData, type RoomUpdateData, type RoomClosedData,
    type FarmState, type GameOverData, HARVEST_ROUNDS,
} from './farm-types';

export type UiPhase = 'connecting' | 'lobby' | 'room' | 'playing';

const KNOWN_ERR = new Set([
    'bad_action', 'not_in_game', 'not_your_turn', 'locked', 'occupied', 'game_over',
    'no_space', 'no_materials', 'no_wood', 'no_seeds', 'not_enough_fields', 'no_room',
    'max_house', 'bad_pasture',
    'already_committed', 'bad_room_id', 'room_not_found', 'room_full', 'already_started',
    'not_in_room', 'not_host', 'cannot_start',
    // 房間數上限（common/room.rs MAX_ROOMS）、連線收訊超量、games 功能被關閉
    'too_many_rooms', 'rate_limited', 'feature_disabled',
]);

/// 對局事件（前端由前後兩則 state 推得，server 不另發）。`seq` = 產生它的那則 state 序號，
/// 同一則 state 推出的事件 seq 相同 —— 語音只念最新一批（`seq === stateSeq`）。
export type FarmEvent =
    | { seq: number; kind: 'action'; round: number; seat: number; action: string; amount: number }
    | { seq: number; kind: 'harvest'; round: number; begging: number[] } // begging = 各家本次新增乞討
    | { seq: number; kind: 'round'; round: number };

const LOG_MAX = 80;

function diffEvents(prev: FarmState | null, next: FarmState, seq: number): FarmEvent[] {
    if (!prev) return [{ seq, kind: 'round', round: next.round }];
    const out: FarmEvent[] = [];
    // 每則 state 都對應一次放置（開局那則 last_action 為 null，已由上方處理）
    if (next.last_action) out.push({ seq, kind: 'action', round: prev.round, ...next.last_action });
    const roundEnded = next.round > prev.round || (next.phase === 'game_over' && prev.phase !== 'game_over');
    if (roundEnded && HARVEST_ROUNDS.has(prev.round)) {
        out.push({ seq, kind: 'harvest', round: prev.round, begging: next.players.map((p, i) => p.begging - (prev.players[i]?.begging ?? 0)) });
    }
    if (next.round > prev.round) out.push({ seq, kind: 'round', round: next.round });
    return out;
}

export interface UseFarmRoom {
    uiPhase: UiPhase;
    rooms: RoomSummary[];
    room: RoomUpdateData | null;
    state: FarmState | null;
    gameOver: GameOverData | null;
    notice: string | null;
    iAmHost: boolean;
    mySeat: number | null;  // 後端逐人注入 your_seat（state 優先，房內用 room_update）
    log: FarmEvent[];
    stateSeq: number;       // 目前 state 的序號（配 FarmEvent.seq 找最新一批）
    actions: {
        createRoom: (roomName: string, nickname: string) => void;
        joinRoom: (roomId: number, nickname: string) => void;
        startGame: () => void;
        sendAction: (action: string, input?: Record<string, unknown>) => void;
        backToLobby: () => void;
    };
}

export function useFarmRoom(): UseFarmRoom {
    const { send } = useWsContext();

    const [uiPhase, setUiPhase] = useState<UiPhase>('connecting');
    const [rooms, setRooms] = useState<RoomSummary[]>([]);
    const [room, setRoom] = useState<RoomUpdateData | null>(null);
    const [state, setState] = useState<FarmState | null>(null);
    const [gameOver, setGameOver] = useState<GameOverData | null>(null);
    const [log, setLog] = useState<FarmEvent[]>([]);
    const [stateSeq, setStateSeq] = useState(0);
    // 上一則 state 與序號只給 WS handler 推事件用，不參與 render
    const prevState = useRef<FarmState | null>(null);
    const seqRef = useRef(0);

    // 房內狀態全清（room_closed / 重連 / 回大廳共用）
    const resetRoom = useCallback(() => {
        setRoom(null); setState(null); setGameOver(null);
        setLog([]); prevState.current = null;
    }, []);

    const { notice, setNotice } = useRoomBase({
        game: GAME,
        knownErrors: KNOWN_ERR,
        handlers: {
            room_list: d => {
                setRooms((d as RoomListData).rooms);
                setUiPhase(p => (p === 'connecting' ? 'lobby' : p));
            },
            lobby_update: d => setRooms((d as RoomListData).rooms),
            room_created: () => { /* room_update 隨後到 */ },
            room_update: d => {
                setRoom(d as RoomUpdateData);
                setUiPhase(p => (p === 'playing' ? p : 'room'));
            },
            room_closed: d => {
                setNotice(`closed_${(d as RoomClosedData).reason}`);
                resetRoom();
                send('join_lobby', undefined, GAME);
                setUiPhase('connecting');
            },
            state: d => {
                const next = d as FarmState;
                const seq = ++seqRef.current;
                const events = diffEvents(prevState.current, next, seq);
                prevState.current = next;
                if (events.length) setLog(l => [...l, ...events].slice(-LOG_MAX));
                setStateSeq(seq);
                setState(next);
                setGameOver(null);
                setUiPhase('playing');
            },
            game_over: d => setGameOver(d as GameOverData),
        },
        // 房間/對局在 server 重啟後已不存在，故清掉房內狀態並回 connecting，等 room_list 重建畫面
        onReconnectReset: () => {
            // 對局 / 房間不支援重連（server 端斷線即中止）：明講，不要無聲跳回大廳
            if (uiPhase === 'room' || uiPhase === 'playing') setNotice('closed_disconnected');
            resetRoom(); setUiPhase('connecting');
        },
        // 切離頁面：在房內 / 對局中要主動退出
        leaveOnUnmount: () => {
            if (uiPhase === 'room' || uiPhase === 'playing') send('leave_room', undefined, GAME);
        },
    });

    const actions = {
        createRoom: useCallback((roomName: string, nickname: string) => {
            setNotice(null);
            const data: Record<string, unknown> = {};
            if (roomName) data.room_name = roomName;
            if (nickname) data.nickname = nickname;
            send('create_room', data, GAME);
        }, [send, setNotice]),
        joinRoom: useCallback((roomId: number, nickname: string) => {
            setNotice(null);
            send('join_room', nickname ? { room_id: roomId, nickname } : { room_id: roomId }, GAME);
        }, [send, setNotice]),
        startGame: useCallback(() => send('start_game', undefined, GAME), [send]),
        sendAction: useCallback((action: string, input?: Record<string, unknown>) => {
            setNotice(null);
            send('action', input ? { action, input } : { action }, GAME);
        }, [send, setNotice]),
        backToLobby: useCallback(() => {
            setNotice(null);
            resetRoom();
            send('leave_room', undefined, GAME);
            send('join_lobby', undefined, GAME);
            setUiPhase('connecting');
        }, [send, setNotice, resetRoom]),
    };

    // your_seat 由後端逐人注入（state 優先，房內用 room_update）
    const mySeat = state?.your_seat ?? room?.your_seat ?? null;
    const iAmHost = room ? room.your_seat === room.host_seat : false;

    return { uiPhase, rooms, room, state, gameOver, notice, iAmHost, mySeat, log, stateSeq, actions };
}

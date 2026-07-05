"use client";

import { useCallback, useState } from 'react';
import { useWsContext } from '@/libs/ws-context';
import { useRoomBase } from '../_shared/useRoomBase';
import {
    GAME, type RoomSummary, type RoomListData, type RoomUpdateData, type RoomClosedData,
    type FarmState, type GameOverData,
} from './farm-types';

export type UiPhase = 'connecting' | 'lobby' | 'room' | 'playing';

const KNOWN_ERR = new Set([
    'bad_action', 'not_in_game', 'not_your_turn', 'locked', 'occupied', 'game_over',
    'no_space', 'no_materials', 'no_wood', 'no_seeds', 'not_enough_fields', 'no_room',
    'max_house', 'bad_pasture',
    'already_committed', 'bad_room_id', 'room_not_found', 'room_full', 'already_started',
    'not_in_room', 'not_host', 'cannot_start',
]);

export interface UseFarmRoom {
    uiPhase: UiPhase;
    rooms: RoomSummary[];
    room: RoomUpdateData | null;
    state: FarmState | null;
    gameOver: GameOverData | null;
    notice: string | null;
    iAmHost: boolean;
    mySeat: number | null;  // 後端逐人注入 your_seat（state 優先，房內用 room_update）
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

    // 房內狀態全清（room_closed / 重連 / 回大廳共用）
    const resetRoom = useCallback(() => {
        setRoom(null); setState(null); setGameOver(null);
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
                setState(d as FarmState);
                setGameOver(null);
                setUiPhase('playing');
            },
            game_over: d => setGameOver(d as GameOverData),
        },
        // 房間/對局在 server 重啟後已不存在，故清掉房內狀態並回 connecting，等 room_list 重建畫面
        onReconnectReset: () => { resetRoom(); setUiPhase('connecting'); },
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

    return { uiPhase, rooms, room, state, gameOver, notice, iAmHost, mySeat, actions };
}

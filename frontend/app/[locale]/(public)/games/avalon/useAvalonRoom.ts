"use client";

import { useCallback, useRef, useState } from 'react';
import { useWsContext } from '@/libs/ws-context';
import { useRoomBase } from '../_shared/useRoomBase';
import {
    GAME,
    type RoomSummary, type RoomListData, type RoomUpdateData, type RoomClosedData,
    type RoleAssignedData, type PhaseChangedData, type TeamProposedData, type VoteResultData,
    type QuestResultData, type GameOverData, type ChatData, type ChatEntry,
    type RoomOptions,
} from './avalon-types';

export type UiPhase = 'connecting' | 'lobby' | 'room' | 'playing';

const KNOWN_ERR = new Set([
    'already_committed', 'bad_room_id', 'room_not_found', 'room_full', 'already_started',
    'not_in_room', 'not_host', 'cannot_start', 'not_in_game', 'not_leader', 'bad_team',
    'bad_team_size', 'wrong_phase', 'bad_vote', 'bad_card', 'not_on_team', 'good_must_succeed',
    'bad_target', 'not_assassin', 'too_many_special_evil', 'bad_player_count',
]);

export interface UseAvalonRoom {
    uiPhase: UiPhase;
    rooms: RoomSummary[];
    room: RoomUpdateData | null;
    role: RoleAssignedData | null;  // ⚠️ 只屬於自己，永不外送
    gamePhase: PhaseChangedData | null;
    proposedTeam: { team: number[]; leader: number } | null;
    voteResult: VoteResultData | null;
    questResult: QuestResultData | null;
    gameOver: GameOverData | null;
    chat: ChatEntry[];
    notice: string | null;          // i18n key（Avalon namespace）
    voted: boolean;
    cardPlayed: boolean;
    iAmHost: boolean;               // 建房者 = host（seat 要到 role_assigned 才知，故本地記）
    actions: {
        createRoom: (roomName: string, nickname: string, options: RoomOptions) => void;
        joinRoom: (roomId: number, nickname: string) => void;
        leaveRoom: () => void;
        startGame: () => void;
        sendChat: (text: string) => void;
        proposeTeam: (team: number[]) => void;
        teamVote: (approve: boolean) => void;
        questCard: (success: boolean) => void;
        assassinate: (target: number) => void;
        backToLobby: () => void;
    };
}

export function useAvalonRoom(): UseAvalonRoom {
    const { send } = useWsContext();

    const [uiPhase, setUiPhase] = useState<UiPhase>('connecting');
    const [rooms, setRooms] = useState<RoomSummary[]>([]);
    const [room, setRoom] = useState<RoomUpdateData | null>(null);
    const [role, setRole] = useState<RoleAssignedData | null>(null);
    const [gamePhase, setGamePhase] = useState<PhaseChangedData | null>(null);
    const [proposedTeam, setProposedTeam] = useState<{ team: number[]; leader: number } | null>(null);
    const [voteResult, setVoteResult] = useState<VoteResultData | null>(null);
    const [questResult, setQuestResult] = useState<QuestResultData | null>(null);
    const [gameOver, setGameOver] = useState<GameOverData | null>(null);
    const [chat, setChat] = useState<ChatEntry[]>([]);
    const [voted, setVoted] = useState(false);
    const [cardPlayed, setCardPlayed] = useState(false);
    const [iAmHost, setIAmHost] = useState(false);

    const chatSeq = useRef(0);

    // 房內狀態全清（room_closed / 重連 / 回大廳共用）
    const resetRoom = useCallback(() => {
        setRoom(null); setRole(null); setGamePhase(null); setGameOver(null); setIAmHost(false);
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
            room_created: () => { /* room_update 隨後到，狀態由它建立 */ },
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
            role_assigned: d => {
                setRole(d as RoleAssignedData);
                setProposedTeam(null); setVoteResult(null); setQuestResult(null);
                setGameOver(null); setVoted(false); setCardPlayed(false);
                setUiPhase('playing');
            },
            phase_changed: d => {
                setGamePhase(d as PhaseChangedData);
                setVoted(false); setCardPlayed(false);
            },
            team_proposed: d => {
                setProposedTeam(d as TeamProposedData);
                setVoteResult(null); setQuestResult(null);
            },
            vote_result: d => setVoteResult(d as VoteResultData),
            quest_result: d => setQuestResult(d as QuestResultData),
            game_over: d => setGameOver(d as GameOverData),
            chat: d => {
                const c = d as ChatData;
                setChat(prev => [...prev, { ...c, id: ++chatSeq.current }].slice(-200));
            },
        },
        // 房間/對局在 server 重啟後已不存在，故清掉房內狀態並回 connecting，等 room_list 重建畫面
        onReconnectReset: () => { resetRoom(); setUiPhase('connecting'); },
        // 切離頁面：在房內 / 對局中要主動退出
        leaveOnUnmount: () => {
            if (uiPhase === 'room' || uiPhase === 'playing') send('leave_room', undefined, GAME);
        },
    });

    const actions = {
        createRoom: useCallback((roomName: string, nickname: string, options: RoomOptions) => {
            setNotice(null);
            const data: Record<string, unknown> = {};
            if (roomName) data.room_name = roomName;
            if (nickname) data.nickname = nickname;
            if (options.mordred || options.oberon) data.options = options;
            setIAmHost(true);
            send('create_room', data, GAME);
        }, [send, setNotice]),
        joinRoom: useCallback((roomId: number, nickname: string) => {
            setNotice(null);
            setIAmHost(false);
            send('join_room', nickname ? { room_id: roomId, nickname } : { room_id: roomId }, GAME);
        }, [send, setNotice]),
        leaveRoom: useCallback(() => { send('leave_room', undefined, GAME); }, [send]),
        startGame: useCallback(() => send('start_game', undefined, GAME), [send]),
        sendChat: useCallback((text: string) => send('chat', { text }, GAME), [send]),
        proposeTeam: useCallback((team: number[]) => send('propose_team', { team }, GAME), [send]),
        teamVote: useCallback((approve: boolean) => { setVoted(true); send('team_vote', { approve }, GAME); }, [send]),
        questCard: useCallback((success: boolean) => { setCardPlayed(true); send('quest_card', { success }, GAME); }, [send]),
        assassinate: useCallback((target: number) => send('assassinate', { target }, GAME), [send]),
        backToLobby: useCallback(() => {
            setNotice(null);
            resetRoom();
            send('leave_room', undefined, GAME);
            send('join_lobby', undefined, GAME);
            setUiPhase('connecting');
        }, [send, setNotice, resetRoom]),
    };

    return {
        uiPhase, rooms, room, role, gamePhase, proposedTeam, voteResult, questResult,
        gameOver, chat, notice, voted, cardPlayed, iAmHost, actions,
    };
}

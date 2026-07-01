"use client";

import { useAvalonRoom } from './useAvalonRoom';
import { AvalonLobby } from './AvalonLobby';
import { AvalonRoom } from './AvalonRoom';
import { AvalonPlay } from './AvalonPlay';

export default function AvalonGame() {
    const room = useAvalonRoom();

    if (room.uiPhase === 'connecting' || room.uiPhase === 'lobby') {
        return (
            <AvalonLobby
                connecting={room.uiPhase === 'connecting'}
                rooms={room.rooms}
                notice={room.notice}
                onCreate={room.actions.createRoom}
                onJoin={room.actions.joinRoom}
            />
        );
    }

    if (room.uiPhase === 'room' && room.room) {
        return (
            <AvalonRoom
                room={room.room}
                iAmHost={room.iAmHost}
                chat={room.chat}
                onStart={room.actions.startGame}
                onLeave={room.actions.backToLobby}
                onChat={room.actions.sendChat}
            />
        );
    }

    if (room.uiPhase === 'playing' && room.role) {
        return <AvalonPlay room={room} />;
    }

    return null;
}

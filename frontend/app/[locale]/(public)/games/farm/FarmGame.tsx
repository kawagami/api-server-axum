"use client";

import { useFarmRoom } from './useFarmRoom';
import { FarmLobby } from './FarmLobby';
import { FarmRoom } from './FarmRoom';
import { FarmPlay } from './FarmPlay';

export default function FarmGame() {
    const farm = useFarmRoom();

    if (farm.uiPhase === 'connecting' || farm.uiPhase === 'lobby') {
        return (
            <FarmLobby
                connecting={farm.uiPhase === 'connecting'}
                rooms={farm.rooms}
                notice={farm.notice}
                onCreate={farm.actions.createRoom}
                onJoin={farm.actions.joinRoom}
            />
        );
    }

    if (farm.uiPhase === 'room' && farm.room) {
        return <FarmRoom room={farm.room} iAmHost={farm.iAmHost} onStart={farm.actions.startGame} onLeave={farm.actions.backToLobby} />;
    }

    if (farm.uiPhase === 'playing' && farm.state) {
        return <FarmPlay farm={farm} />;
    }

    return null;
}

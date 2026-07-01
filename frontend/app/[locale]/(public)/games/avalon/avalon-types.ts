// 阿瓦隆 Avalon wire 型別（唯一契約 docs/2026-06-19-avalon-frontend.md）。
// N 人社交推理，與 2 人對戰框架無關，只共用 /ws + 信封 { game:"avalon", type, data }。

export const GAME = 'avalon';

export type AvalonRole =
    | 'merlin' | 'percival' | 'loyal_servant'      // 好人
    | 'assassin' | 'morgana' | 'mordred' | 'oberon' | 'minion'; // 壞人

export const EVIL_ROLES: ReadonlySet<AvalonRole> = new Set([
    'assassin', 'morgana', 'mordred', 'oberon', 'minion',
]);
export const GOOD_ROLES: ReadonlySet<AvalonRole> = new Set([
    'merlin', 'percival', 'loyal_servant',
]);

export type AvalonPhase = 'team_building' | 'team_vote' | 'quest' | 'assassinate' | 'game_over';

export interface RoomSummary { id: number; name: string; players: number; max: number; status: string; }
export interface PlayerInfo { seat: number; name: string; }
export interface RoomOptions { mordred?: boolean; oberon?: boolean; }

// 下行 data 形狀
export interface RoomListData { rooms: RoomSummary[]; }
export interface RoomCreatedData { room_id: number; }
export interface RoomUpdateData {
    room_id: number; name: string; host_seat: number;
    players: PlayerInfo[]; options: RoomOptions; can_start: boolean;
}
export interface RoomClosedData { reason: string; }
// ⚠️ 逐人私有，絕不外送 / 不渲染給他人
export interface RoleAssignedData {
    your_seat: number; your_role: AvalonRole; known: number[];
    n: number; sizes: number[]; players: PlayerInfo[];
}
export interface PhaseChangedData {
    phase: AvalonPhase; leader: number; round: number; quest_size: number;
    results: boolean[]; rejects: number; team: number[];
}
export interface TeamProposedData { team: number[]; leader: number; }
export interface VoteResultData { votes: { seat: number; approve: boolean }[]; approved: boolean; }
export interface QuestResultData { round: number; fails: number; success: boolean; }
export interface GameOverData { winner: 'good' | 'evil' | null; reason: string; roles: { seat: number; role: AvalonRole }[]; }
export interface ChatData { seat: number; name: string; text: string; }
export interface ChatEntry extends ChatData { id: number; } // 前端加流水號當 key
export interface ErrorData { reason: string; }

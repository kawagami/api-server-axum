// 農場經營 Farm wire 型別（唯一契約 docs/2026-06-19-farm-frontend.md）。
// 2–4 人完全資訊 worker-placement。每次 action 後 server 廣播完整 state，前端整盤重繪。
// 信封 { game:"farm", type, data }。盤面抽象（無逐格座標，phase-1）。

export const GAME = 'farm';

export type HouseKind = 'wood' | 'clay' | 'stone';
export type Crop = 'grain' | 'veg';
export type AnimalKind = 'sheep' | 'boar' | 'cattle';

export interface FieldPlot { crop: Crop; count: number; } // null = 已犁未播
export interface Pasture { tiles: number; stable: boolean; animal: { kind: AnimalKind; count: number } | null; }

export interface PlayerState {
    house: HouseKind; rooms: number; family: number;
    fields: (FieldPlot | null)[];
    pastures: Pasture[];
    loose_stables: number; free_tiles: number;
    wood: number; clay: number; reed: number; stone: number; grain: number; veg: number;
    sheep: number; boar: number; cattle: number; food: number; begging: number;
}

export interface FarmState {
    your_seat: number;  // 逐人注入：收訊者自己的 seat
    round: number;
    phase: 'placing' | 'game_over';
    current_player: number | null;
    starting_player: number;
    available_actions: string[];
    accumulation: { action: string; amount: number }[];
    players: PlayerState[];
}

export interface GameOverData { scores: number[]; }

// 收穫輪（第 4/7/9/11/13/14 輪末），round 從 1 起算
export const HARVEST_ROUNDS = new Set([4, 7, 9, 11, 13, 14]);

// 累積格（值會堆積，取走全拿）
export const ACCUMULATION_ACTIONS = new Set([
    'forest', 'clay_pit', 'reed', 'quarry', 'river',
    'sheep_pen', 'boar_pen', 'cattle_pen',
]);
// 需 input 表單的複合動作
export const COMPOUND_ACTIONS = new Set(['sow', 'build_rooms', 'fences']);

// 房 / 大廳 wire（與 avalon 同形）
export interface RoomSummary { id: number; name: string; players: number; max: number; status: string; }
export interface PlayerInfo { seat: number; name: string; }
export interface RoomListData { rooms: RoomSummary[]; }
export interface RoomCreatedData { room_id: number; }
export interface RoomUpdateData { room_id: number; name: string; host_seat: number; players: PlayerInfo[]; can_start: boolean; your_seat: number; }
export interface RoomClosedData { reason: string; }
export interface ErrorData { reason: string; }

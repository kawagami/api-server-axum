// 對戰遊戲 WS 共用 wire 型別（唯一契約來源 docs/chess-wire-protocol.md）。
// 大廳/桌位/配對/計時/斷線三遊戲共用；各遊戲的 move/move_made 內容在各自 logic 檔。
// 信封 { game, type, data }，game 必填（'chess'/'gomoku'/'banqi'）。

export type GameId = 'chess' | 'gomoku' | 'banqi' | 'western_chess' | 'go';

export type TableStatus = 'waiting' | 'playing';
export interface WireTable { id: number; name: string; status: TableStatus; }

export interface TableListData { tables: WireTable[]; }
export interface TableCreatedData { table_id: number; }
export interface QueuedData { position: number; }
// match_found.color = 我方 color/seat 標籤（chess red/black、gomoku black/white、banqi first/second）
export interface MatchFoundData { color: string; clock_ms: number; table_id: number; }
// move_made 外殼三遊戲共用：turn=下一手輪到誰，clock 鍵=兩方標籤、值=剩餘 ms；其餘欄位各遊戲自有
export interface MoveMadeBase { turn: string; clock: Record<string, number>; }
export interface GameOverData { winner: string | null; reason: string; }
export interface IllegalMoveData { reason: string; }
export interface ErrorData { reason: string; }

// hints：server 推的合法步提示（唯讀，**非權威** —— 判定仍只在後端 try_move）。
// 只送給當前輪到的一方，形狀依遊戲不同：
//   象棋 / 西洋棋 / 暗棋 → moves（key = "col,row" 的來源格）；暗棋另有 flips（可翻的格）
//   圍棋 → forbidden（空點裡不能下的：自殺 / 劫）
//   五子棋不送（合法點 = 所有空點，前端自己知道）
export type HintCell = [number, number];
export interface HintsData {
    moves?: Record<string, HintCell[]>;
    flips?: HintCell[];
    forbidden?: HintCell[];
}

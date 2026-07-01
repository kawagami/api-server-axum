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

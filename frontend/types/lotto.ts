// 大樂透 / 威力彩 選號登錄 + 自動對獎（POST /member/lotto；與發票對獎為不同功能）
export type LottoGame = 'lotto649' | 'super_lotto638';
export type LottoSource = 'qr' | 'manual';
export type LottoStatus = 'pending' | 'won' | 'lost';

// 中獎獎別 key；須搭配 game 解讀（lotto649 到 seventh+general、super_lotto638 到 ninth）
export type LottoPrizeTier =
  | 'first'
  | 'second'
  | 'third'
  | 'fourth'
  | 'fifth'
  | 'sixth'
  | 'seventh'
  | 'eighth'
  | 'ninth'
  | 'general';

// 一注：一般號/第一區 6 個相異；威力彩第二區 second(1-8)，大樂透 second 為 null
export interface LottoNote {
  picks: number[];
  second: number | null;
}

export interface LottoTicket {
  id: string;
  member_id: number;
  game: LottoGame;
  draw_date: string; // 這注要對的開獎日 YYYY-MM-DD
  picks: number[];
  second: number | null;
  source: LottoSource;
  checked: boolean; // 該期是否已開獎並對過
  prize_tier: LottoPrizeTier | null; // checked=true 且為 null = 確定未中
  notified_at: string | null;
  created_at: string;
  updated_at: string;
}

// 批次登錄（POST /member/lotto）：整批共用 game/draw_date/source，notes 帶多注
export interface LottoInput {
  game: LottoGame;
  draw_date: string;
  source: LottoSource;
  notes: LottoNote[];
}

export interface LottoListParams {
  game?: LottoGame;
  status?: LottoStatus; // pending=未開獎、won=中獎、lost=未中、省略=全部
  page?: number;
  per_page?: number;
}

export interface LottoDraw {
  game: LottoGame;
  period: string; // 台彩期別字串，資訊用
  draw_date: string;
  main_nums: number[]; // 一般號/第一區（已排序）
  special: number; // 大樂透=特別號；威力彩=第二區號
}

export interface LottoDrawParams {
  game?: LottoGame;
  limit?: number;
}

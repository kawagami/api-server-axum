// Food log（後端 structs/food_log.rs）

/** 後端 `Meal` enum；顯示順序見 `food-log/model.ts` 的 `MEALS` */
export type Meal = 'breakfast' | 'lunch' | 'dinner' | 'snack' | 'late_night' | 'other';

export interface FoodLogEntry {
  id: number;
  eaten_on: string;
  meal: Meal;
  meal_label: string | null;
  store: string | null;
  item: string;
  quantity: number;
  /** 該行總價；null = 點數兌換 / 請客 */
  amount: number | null;
  rating: number | null;
  note: string | null;
  created_at: string;
  updated_at: string;
}

/** POST / PUT 共用（PUT 是整筆覆寫）。選填欄位送空字串 = 沒填，後端收成 null */
export interface FoodLogEntryInput {
  eaten_on: string;
  meal: Meal;
  meal_label?: string | null;
  store?: string | null;
  item: string;
  quantity?: number;
  amount?: number | null;
  rating?: number | null;
  note?: string | null;
}

export interface FoodLogDay {
  date: string;
  note: string | null;
  /** 有標金額的品項加總 */
  total: number;
  /** 已依餐別、輸入順序排好 */
  entries: FoodLogEntry[];
}

/** 同一個 (store, item) 一列，帶最近一次的數量與金額供一鍵帶入 */
export interface FoodLogSuggestion {
  store: string | null;
  item: string;
  last_quantity: number;
  last_amount: number | null;
  last_eaten_on: string;
  count: number;
  meal_counts: Record<Meal, number>;
}

/** 台北日界；week 從週一起算 */
export interface FoodLogSummary {
  today: number;
  week: number;
  month: number;
}

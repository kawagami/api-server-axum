import type { Log } from "@/types";

/** 連續重複的 log 合併成一列的時間窗（retry 這類事件常常一秒內連噴好幾筆） */
const DUPE_WINDOW_MS = 5 * 60 * 1000;

/**
 * fields 的顯示順序。`self` 一定排第一 —— 那是真正的錯誤原因
 * （message 只是 `System error occurred` 這種固定字串），其餘照請求上下文的閱讀順序。
 * 不在清單裡的 key 依字母序接在後面，所以新增 span field 不必改這裡。
 */
const FIELD_ORDER = ['self', 'panic', 'method', 'path', 'query', 'ip', 'status', 'latency_ms'];

export function sortedFields(fields: Record<string, unknown>): [string, string][] {
    return Object.entries(fields)
        .map(([k, v]): [string, string] => [k, typeof v === 'string' ? v : JSON.stringify(v)])
        .sort(([a], [b]) => {
            const ia = FIELD_ORDER.indexOf(a);
            const ib = FIELD_ORDER.indexOf(b);
            if (ia !== -1 && ib !== -1) return ia - ib;
            if (ia !== -1) return -1;
            if (ib !== -1) return 1;
            return a.localeCompare(b);
        });
}

/** 一列 = 一個事件；連續重複的原始 log 收在 rows 裡（head 是最新那筆） */
export interface LogGroup {
    head: Log;
    rows: Log[];
}

/**
 * 把**相鄰**且同層級／同來源／同訊息、時間相差在 DUPE_WINDOW_MS 內的 log 合併成一列。
 *
 * 只合併相鄰的（清單是新→舊），所以不會把中間夾著別的事件的兩筆黏在一起。
 * 合併掉的筆數不會消失 —— 列上標 `×N`，展開面板逐筆列出時間與 request_id；
 * 面板上方的 fields 一律取最新那筆。
 */
export function groupConsecutive(logs: Log[]): LogGroup[] {
    const groups: LogGroup[] = [];
    for (const log of logs) {
        const last = groups[groups.length - 1];
        const prev = last?.rows[last.rows.length - 1];
        const sameEvent =
            last && last.head.level === log.level && last.head.target === log.target && last.head.message === log.message;
        const withinWindow =
            prev && Math.abs(new Date(prev.created_at).getTime() - new Date(log.created_at).getTime()) <= DUPE_WINDOW_MS;
        if (sameEvent && withinWindow) {
            last.rows.push(log);
        } else {
            groups.push({ head: log, rows: [log] });
        }
    }
    return groups;
}

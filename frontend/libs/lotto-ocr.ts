import type { LottoGame, LottoNote } from "@/types";
import { GAME_SPECS } from "@/libs/lotto";

// OCR 解析結果（純函式，零 DOM，可測）。OCR 僅輔助，結果一律進可編輯的選號區由使用者核對
export interface OcrParseResult {
    drawDate: string | null; // 西元 YYYY-MM-DD（民國年 +1911）
    notes: LottoNote[]; // 偵測到的注（第二區一律 null，威力彩請使用者補）
    detectedLines: number; // 偵測到的注行數（含後續被判無效者）
}

// 民國日期 → 西元 YYYY-MM-DD（115/06/23 → 2026-06-23）
function rocToIso(y: number, m: number, d: number): string | null {
    if (m < 1 || m > 12 || d < 1 || d > 31) return null;
    const year = y + 1911;
    return `${year}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
}

// 排除非選號行（日期/期別/金額/機台等）
const META_RE = /開獎|期別|銷售|投注|經銷|總金額|計\s*\d+\s*期|WID/i;

export function parseLottoOcr(text: string, game: LottoGame): OcrParseResult {
    const spec = GAME_SPECS[game];
    const lines = text.split(/\r?\n/).map(l => l.trim()).filter(Boolean);

    // 開獎日：抓民國日期（2-3 碼年/月/日）。OCR 數字白名單會濾掉中文，故「開獎」字樣常不在；
    // 票面開獎日期印在銷售日期之上，取第一個日期即為開獎日（含「開獎」時優先）
    const dateRe = /(\d{2,3})\s*[/／.\-]\s*(\d{1,2})\s*[/／.\-]\s*(\d{1,2})/;
    let drawDate: string | null = null;
    const dateLine = lines.find(l => /開獎/.test(l) && dateRe.test(l)) ?? lines.find(l => dateRe.test(l));
    const dm = dateLine?.match(dateRe);
    if (dm) drawDate = rocToIso(+dm[1], +dm[2], +dm[3]);

    // 注：彩券選號一律印成 2 位數補零（01 04 06…）。先排除日期/金額行，再抓 2 位數 token，
    // 落在 1..mainMax、去重後「恰」mainCount 個才算一注（寧可漏抓也不誤抓 → 避免序號/WID 行誤判）
    const notes: LottoNote[] = [];
    for (const line of lines) {
        if (META_RE.test(line)) continue;
        if (dateRe.test(line)) continue; // 跳過開獎/銷售日期行
        const cleaned = line.replace(/\$\s*\d+/g, ' ').replace(/NT/gi, ' ');
        // 只取「恰 2 位數」的整段數字（票面選號補零成 01..49）；序號/WID 是長段數字，自然被排除
        const runs: string[] = cleaned.match(/\d+/g) ?? [];
        const nums = runs.filter(r => r.length === 2).map(Number).filter(n => n >= 1 && n <= spec.mainMax);
        const uniq = [...new Set(nums)];
        if (uniq.length === spec.mainCount) {
            notes.push({ picks: uniq.sort((a, b) => a - b), second: null });
        }
    }

    return { drawDate, notes, detectedLines: notes.length };
}

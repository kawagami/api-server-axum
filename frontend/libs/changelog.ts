// 更新紀錄（/changelog）的解析規則單一來源。
// 資料來源是 GitHub commits API（api/github.ts），message 是 Conventional Commits 格式，
// 這裡把 `feat(games): xxx` 拆成 type / scope / subject，並決定哪些 type 要見光。
//
// 只列「使用者看得到差別」的 type：chore / ci / docs / test / build / style 與 dependabot 的
// deps 全部濾掉（歷史上 chore 47 筆、docs 21 筆，全列會把真正的功能更新洗掉）。
// scope 為 deps 的也濾（`chore(deps)` 已被 type 濾掉，但 `fix(deps)` 同樣是升版雜訊）。
//
// 不用語意色（紅/綠/橘在本站有既定意義，見 CLAUDE.md「語意色保留」），
// type 一律 primary chip，靠 icon 區分。

export const CHANGELOG_TYPES = ["feat", "fix", "perf", "refactor", "security"] as const;

export type ChangelogType = (typeof CHANGELOG_TYPES)[number];

export interface ChangelogEntry {
    /** 短 sha（7 碼），顯示與 React key 用 */
    sha: string;
    /** GitHub commit 頁面連結 */
    url: string;
    /** commit 時間，ISO 字串 */
    date: string;
    type: ChangelogType;
    /** `feat(games):` 的 games，沒有 scope 時為 undefined */
    scope?: string;
    subject: string;
    /** `feat!:` / `feat(x)!:` 的破壞性變更標記 */
    breaking: boolean;
}

const SUBJECT_RE = /^([a-z]+)(?:\(([^)]*)\))?(!)?:\s*(.+)$/;

const VISIBLE = new Set<string>(CHANGELOG_TYPES);

/**
 * 解析 commit message 第一行。不是 Conventional 格式（merge commit、subtree add）
 * 或 type/scope 在濾除清單裡就回 null —— 呼叫端據此丟棄。
 */
export function parseCommitSubject(message: string): Omit<ChangelogEntry, "sha" | "url" | "date"> | null {
    const first = message.split("\n", 1)[0].trim();
    const m = SUBJECT_RE.exec(first);
    if (!m) return null;

    const [, type, scope, bang, subject] = m;
    if (!VISIBLE.has(type)) return null;
    if (scope === "deps") return null;

    return {
        type: type as ChangelogType,
        scope: scope || undefined,
        subject,
        breaking: Boolean(bang),
    };
}

/** 依日期（當地時區的 YYYY-MM-DD）分組，順序沿用傳入陣列（GitHub 回的是新到舊） */
export function groupByDay(entries: ChangelogEntry[]): { day: string; items: ChangelogEntry[] }[] {
    const groups: { day: string; items: ChangelogEntry[] }[] = [];
    for (const entry of entries) {
        const day = entry.date.slice(0, 10);
        const last = groups[groups.length - 1];
        if (last && last.day === day) last.items.push(entry);
        else groups.push({ day, items: [entry] });
    }
    return groups;
}

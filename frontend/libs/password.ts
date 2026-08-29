/**
 * 密碼產生的單一來源。**刻意跑在瀏覽器**，不打後端。
 *
 * 原本這頁是 `GET /tools/new_password`（Rust 端 `rand::rng()` 產生後回傳），品質沒問題，
 * 問題在傳輸：一組明文密碼要走 瀏覽器 → Next server action → nginx → Rust 三跳、
 * 經過兩台機器的記憶體，還可能落在任何一層的存取紀錄裡。密碼沒有任何理由離開產生它的裝置。
 * 改成 `crypto.getRandomValues` 後順帶：零延遲、離線可用、不吃 1 核 1G VPS 的 tools rate limit。
 * 後端那支端點已隨之移除（唯一呼叫端就是這頁）。
 */

export const LOWERCASE = "abcdefghijklmnopqrstuvwxyz";
export const UPPERCASE = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
export const DIGITS = "0123456789";
/** 鍵盤上不必按 AltGr、貼到大多數系統都不會被吃掉的一組 */
export const SYMBOLS = "!@#$%^&*()-_=+[]{};:,.?/";

/** 手抄或唸給別人聽時最容易搞混的字元 */
export const AMBIGUOUS = "0O1lI|";

export const CHARSET_KEYS = ["lowercase", "uppercase", "digits", "symbols"] as const;
export type CharsetKey = (typeof CHARSET_KEYS)[number];

const CHARSETS: Record<CharsetKey, string> = {
    lowercase: LOWERCASE,
    uppercase: UPPERCASE,
    digits: DIGITS,
    symbols: SYMBOLS,
};

export interface PasswordOptions {
    length: number;
    charsets: Record<CharsetKey, boolean>;
    excludeAmbiguous: boolean;
}

export const COUNT_MIN = 1;
export const COUNT_MAX = 20;
export const LENGTH_MIN = 4;
export const LENGTH_MAX = 128;

export function clampCount(n: number) {
    return clamp(n, COUNT_MIN, COUNT_MAX);
}

export function clampLength(n: number) {
    return clamp(n, LENGTH_MIN, LENGTH_MAX);
}

function clamp(n: number, min: number, max: number) {
    if (!Number.isFinite(n)) return min;
    return Math.min(Math.max(Math.trunc(n), min), max);
}

/** 依勾選的類別組出可用字元池。回傳每個類別各自的池，強制「每類至少一個」時要用到 */
export function buildPools(options: PasswordOptions): string[] {
    return CHARSET_KEYS.filter((key) => options.charsets[key])
        .map((key) => (options.excludeAmbiguous ? stripAmbiguous(CHARSETS[key]) : CHARSETS[key]))
        .filter((pool) => pool.length > 0);
}

function stripAmbiguous(charset: string) {
    return [...charset].filter((c) => !AMBIGUOUS.includes(c)).join("");
}

/**
 * 無偏的隨機索引。**不要寫成 `getRandomValues()[0] % n`** —— 2³² 不是 n 的倍數，
 * 尾端那段不完整的區間會讓前幾個字元的機率略高（modulo bias）。
 * 這裡用 rejection sampling：落在不完整區間的值直接丟掉重抽。
 */
function randomIndices(count: number, n: number): number[] {
    const limit = Math.floor(4294967296 / n) * n;
    const out: number[] = [];
    // 抓一批再逐個消化，避免每個字元都呼叫一次 getRandomValues
    const buf = new Uint32Array(Math.max(count, 16));
    while (out.length < count) {
        crypto.getRandomValues(buf);
        for (const v of buf) {
            if (v >= limit) continue;
            out.push(v % n);
            if (out.length === count) break;
        }
    }
    return out;
}

function randomChar(pool: string) {
    return pool[randomIndices(1, pool.length)[0]];
}

/** Fisher–Yates，亂數同樣走 CSPRNG（`Math.random()` 不是密碼學安全的） */
function shuffle(chars: string[]) {
    for (let i = chars.length - 1; i > 0; i--) {
        const j = randomIndices(1, i + 1)[0];
        [chars[i], chars[j]] = [chars[j], chars[i]];
    }
    return chars;
}

/**
 * 產生一組密碼。勾選的每個類別都保證至少出現一次（長度夠的話），
 * 免得勾了符號卻抽出一組純字母、貼到有「必須含符號」規則的網站被退。
 *
 * 代價是字元頻率不再完全均勻：保證字是「每類各抽一個」，於是小池的字元被多抽到
 * ——四類全開、長度 32 時數字約占 13.3%（純均勻是 10/86 ≈ 11.6%）。這是所有
 * 「保證含各類」產生器的共同取捨，不是取樣缺陷（取樣本身無偏，見 randomIndices）。
 */
export function generatePassword(options: PasswordOptions): string {
    const pools = buildPools(options);
    if (pools.length === 0) return "";

    const combined = pools.join("");
    const guaranteed = pools.slice(0, options.length).map(randomChar);
    const rest = randomIndices(Math.max(options.length - guaranteed.length, 0), combined.length).map(
        (i) => combined[i],
    );

    return shuffle([...guaranteed, ...rest]).join("");
}

export function generatePasswords(count: number, options: PasswordOptions): string[] {
    return Array.from({ length: count }, () => generatePassword(options));
}

/**
 * 熵（bits）＝ 長度 × log2(字元池大小)。
 *
 * 嚴格說「每類至少一個」的約束讓真實熵略低於這個值（少掉的量在 1 bit 以內，
 * 長度越長越可忽略），業界慣例都報這個上界，這裡跟進，不做無謂的精算。
 */
export function passwordEntropyBits(options: PasswordOptions): number {
    const size = buildPools(options).join("").length;
    if (size < 2) return 0;
    return options.length * Math.log2(size);
}

export type StrengthLevel = "weak" | "fair" | "good" | "strong";

/**
 * 分級門檻取自「離線破解」的角度：45 bits 以下現代 GPU 叢集是小時級，
 * 65 起才談得上成本，90 以上在可見的未來都不划算。
 */
export function strengthLevel(bits: number): StrengthLevel {
    if (bits < 45) return "weak";
    if (bits < 65) return "fair";
    if (bits < 90) return "good";
    return "strong";
}

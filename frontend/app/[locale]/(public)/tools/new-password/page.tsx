"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslations } from "next-intl";
import { Check, Copy, RefreshCw, ShieldCheck } from "lucide-react";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import Toast, { useToast } from "@/components/toast";
import {
    CHARSET_KEYS,
    COUNT_MAX,
    COUNT_MIN,
    DIGITS,
    LENGTH_MAX,
    LENGTH_MIN,
    SYMBOLS,
    clampCount,
    clampLength,
    generatePasswords,
    passwordEntropyBits,
    strengthLevel,
    type CharsetKey,
    type PasswordOptions,
} from "@/libs/password";

const STRENGTH_STYLE = {
    weak: { bar: "bg-red-500", text: "text-red-600 dark:text-red-400", filled: 1 },
    fair: { bar: "bg-amber-500", text: "text-amber-600 dark:text-amber-400", filled: 2 },
    good: { bar: "bg-primary-400", text: "text-primary-600 dark:text-primary-400", filled: 3 },
    strong: { bar: "bg-primary-600", text: "text-primary-700 dark:text-primary-300", filled: 4 },
} as const;

const DEFAULT_CHARSETS: Record<CharsetKey, boolean> = {
    lowercase: true,
    uppercase: true,
    digits: true,
    symbols: true,
};

export default function NewPasswordPage() {
    const t = useTranslations("NewPassword");
    const [count, setCount] = useState(5);
    const [length, setLength] = useState(16);
    const [charsets, setCharsets] = useState(DEFAULT_CHARSETS);
    const [excludeAmbiguous, setExcludeAmbiguous] = useState(false);
    const [passwords, setPasswords] = useState<string[]>([]);
    const [copiedKey, setCopiedKey] = useState<string | null>(null);
    const { toast, showToast } = useToast();
    const copiedTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

    const options: PasswordOptions = { length, charsets, excludeAmbiguous };
    const hasCharset = CHARSET_KEYS.some((key) => charsets[key]);
    const bits = passwordEntropyBits(options);
    const level = strengthLevel(bits);
    const strength = STRENGTH_STYLE[level];

    const regenerate = useCallback(() => {
        if (!CHARSET_KEYS.some((key) => charsets[key])) {
            setPasswords([]);
            return;
        }
        setPasswords(generatePasswords(count, { length, charsets, excludeAmbiguous }));
        setCopiedKey(null);
    }, [count, length, charsets, excludeAmbiguous]);

    // 進站就先給一批，之後每次調整選項也即時換一批（產生是本機運算，沒有請求成本，
    // 不必讓使用者先按一下才看到東西；調了長度卻還看著舊密碼更怪）。
    // 不能在 render 期間算：`crypto.getRandomValues` 是 client-only，而且亂數在
    // server / client 兩邊必然對不上，會 hydration mismatch。
    // eslint-disable-next-line react-hooks/set-state-in-effect
    useEffect(() => regenerate(), [regenerate]);

    useEffect(() => () => {
        if (copiedTimer.current) clearTimeout(copiedTimer.current);
    }, []);

    const markCopied = (key: string) => {
        setCopiedKey(key);
        if (copiedTimer.current) clearTimeout(copiedTimer.current);
        copiedTimer.current = setTimeout(() => setCopiedKey(null), 2000);
    };

    const copy = async (text: string, key: string, successMessage: string) => {
        try {
            await navigator.clipboard.writeText(text);
            markCopied(key);
            showToast("success", successMessage);
        } catch {
            // clipboard API 在非安全來源（http）與部分舊瀏覽器不存在
            showToast("error", t("copyFail"));
        }
    };

    const toggleCharset = (key: CharsetKey) =>
        setCharsets((prev) => ({ ...prev, [key]: !prev[key] }));

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t("title")} description={t("localOnlyNote")} />

            <section className="flex flex-col gap-5 rounded-lg bg-white dark:bg-neutral-800 p-4 shadow-xs">
                <h2 className="sr-only">{t("optionsHeading")}</h2>

                <SliderField
                    id="length"
                    label={t("lengthLabel")}
                    min={LENGTH_MIN}
                    max={LENGTH_MAX}
                    value={length}
                    onChange={(n) => setLength(clampLength(n))}
                />
                <SliderField
                    id="count"
                    label={t("countLabel")}
                    min={COUNT_MIN}
                    max={COUNT_MAX}
                    value={count}
                    onChange={(n) => setCount(clampCount(n))}
                />

                <fieldset className="flex flex-col gap-2">
                    <legend className="text-sm text-neutral-600 dark:text-neutral-300 mb-1">{t("charsetLabel")}</legend>
                    <div className="flex flex-wrap gap-x-5 gap-y-2">
                        {CHARSET_KEYS.map((key) => (
                            <Checkbox
                                key={key}
                                id={`charset-${key}`}
                                checked={charsets[key]}
                                onChange={() => toggleCharset(key)}
                                label={t(`charset.${key}`)}
                            />
                        ))}
                    </div>
                    <Checkbox
                        id="exclude-ambiguous"
                        checked={excludeAmbiguous}
                        onChange={() => setExcludeAmbiguous((v) => !v)}
                        label={t("excludeAmbiguous")}
                        hint={t("excludeAmbiguousHint")}
                    />
                    {!hasCharset && (
                        <p role="alert" className="text-sm text-red-500 dark:text-red-400">
                            {t("noCharsetSelected")}
                        </p>
                    )}
                </fieldset>

                <div className="flex flex-col gap-1.5">
                    <div className="flex items-center justify-between text-sm">
                        <span className="flex items-center gap-1.5 text-neutral-600 dark:text-neutral-300">
                            <ShieldCheck size={15} />
                            {t("strengthLabel")}
                        </span>
                        <span className={`font-medium ${strength.text}`}>
                            {t(`strength.${level}`)}
                            <span className="ml-2 font-normal text-neutral-500 dark:text-neutral-400">
                                {t("entropy", { bits: Math.round(bits) })}
                            </span>
                        </span>
                    </div>
                    <div className="flex gap-1" role="presentation">
                        {[0, 1, 2, 3].map((i) => (
                            <span
                                key={i}
                                className={`h-1.5 flex-1 rounded-full transition-colors ${
                                    hasCharset && i < strength.filled
                                        ? strength.bar
                                        : "bg-neutral-200 dark:bg-neutral-700"
                                }`}
                            />
                        ))}
                    </div>
                </div>
            </section>

            <div className="flex flex-wrap gap-3">
                <button
                    onClick={regenerate}
                    disabled={!hasCharset}
                    className="flex items-center gap-2 px-4 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 disabled:hover:bg-primary-500 transition-colors"
                >
                    <RefreshCw size={16} />
                    {t("regenerate")}
                </button>
                <button
                    onClick={() => copy(passwords.join("\n"), "__all__", t("copyAllSuccess", { count: passwords.length }))}
                    disabled={passwords.length === 0}
                    className="flex items-center gap-2 px-4 py-2 rounded-md border border-neutral-300 dark:border-neutral-600 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-700 disabled:opacity-50 transition-colors"
                >
                    {copiedKey === "__all__" ? <Check size={16} /> : <Copy size={16} />}
                    {t("copyAll")}
                </button>
            </div>

            <div className="flex flex-col gap-3">
                <h2 className="text-lg font-semibold text-neutral-800 dark:text-neutral-100">{t("resultLabel")}</h2>
                {/* 產生是同步的、沒有 loading 態，但換一批對螢幕閱讀器來說是無聲的，要靠 aria-live 播報 */}
                <div aria-live="polite" aria-atomic="false">
                    {passwords.length > 0 ? (
                        <ul className="flex flex-col gap-2">
                            {passwords.map((password, index) => {
                                const key = `${index}-${password}`;
                                return (
                                    <li
                                        key={key}
                                        className="flex items-center justify-between gap-3 rounded-md bg-white dark:bg-neutral-800 px-3 py-2 shadow-xs"
                                    >
                                        <span className="font-mono text-sm wrap-break-word text-neutral-800 dark:text-neutral-100">
                                            {[...password].map((char, i) => (
                                                <span
                                                    key={i}
                                                    className={
                                                        DIGITS.includes(char) || SYMBOLS.includes(char)
                                                            ? "text-primary-600 dark:text-primary-400"
                                                            : undefined
                                                    }
                                                >
                                                    {char}
                                                </span>
                                            ))}
                                        </span>
                                        <button
                                            onClick={() => copy(password, key, t("copySuccess"))}
                                            aria-label={t("copyOne", { index: index + 1 })}
                                            className="shrink-0 flex items-center gap-1 px-2 py-1 text-sm rounded-sm border border-neutral-200 dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
                                        >
                                            {copiedKey === key ? <Check size={14} /> : <Copy size={14} />}
                                            {copiedKey === key ? t("copied") : t("copy")}
                                        </button>
                                    </li>
                                );
                            })}
                        </ul>
                    ) : (
                        <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("empty")}</p>
                    )}
                </div>
            </div>

            <Toast toast={toast} />
        </PageShell>
    );
}

/**
 * 數量／長度都用滑桿而不是 `type="number"`：數字框在 onChange 就 clamp 會讓「清空再重打」
 * 變成不可能（`Number("")` = 0 → 立刻跳成下限），滑桿沒有中間無效態，鍵盤方向鍵也能精調。
 */
function SliderField({
    id,
    label,
    min,
    max,
    value,
    onChange,
}: {
    id: string;
    label: string;
    min: number;
    max: number;
    value: number;
    onChange: (n: number) => void;
}) {
    return (
        <div className="flex flex-col gap-1">
            <div className="flex items-center justify-between">
                <label htmlFor={id} className="text-sm text-neutral-600 dark:text-neutral-300">
                    {label}
                </label>
                <output htmlFor={id} className="font-mono text-sm tabular-nums text-neutral-800 dark:text-neutral-100">
                    {value}
                </output>
            </div>
            <input
                id={id}
                type="range"
                min={min}
                max={max}
                value={value}
                onChange={(e) => onChange(Number(e.target.value))}
                className="w-full accent-primary-500"
            />
        </div>
    );
}

function Checkbox({
    id,
    checked,
    onChange,
    label,
    hint,
}: {
    id: string;
    checked: boolean;
    onChange: () => void;
    label: string;
    hint?: string;
}) {
    return (
        <label htmlFor={id} className="flex items-center gap-2 text-sm text-neutral-700 dark:text-neutral-200 cursor-pointer">
            <input
                id={id}
                type="checkbox"
                checked={checked}
                onChange={onChange}
                className="size-4 accent-primary-500"
            />
            <span>
                {label}
                {hint && <span className="ml-1.5 font-mono text-xs text-neutral-500 dark:text-neutral-400">{hint}</span>}
            </span>
        </label>
    );
}

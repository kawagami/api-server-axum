"use client";

import { useState, useActionState, useMemo, useRef } from "react";
import { useTranslations } from "next-intl";
import { ArrowLeftRight } from "lucide-react";
import { convertTextAction, type ConvertTextState } from "@/app/[locale]/(public)/tools/convert-text/actions";
import {
    CONVERT_TEXT_MAX_BYTES,
    CONVERT_TEXT_MAX_KB,
    utf8ByteLength,
    type ConversionDirection,
} from "@/libs/convert-text";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import Toast, { useToast } from "@/components/toast";

const DIRECTIONS = [
    { value: "s2t", labelKey: "s2t" },
    { value: "t2s", labelKey: "t2s" },
] as const;

// 沒有 twMerge / cn helper，所以顏色類別不能疊在同一個 class 字串裡：
// 同一個 element 同時出現 bg-white 與 bg-neutral-100（或 border-neutral-300 與 border-red-500）時，
// 誰贏取決於 Tailwind 產出的 CSS 順序，不是這裡的字串順序。base 只留不衝突的部分。
const textareaBase = "w-full p-3 text-base rounded-md border";
const neutralBorder = "border-neutral-300 dark:border-neutral-600";
const inputBg = "bg-white dark:bg-neutral-800";
const resultBg = "bg-neutral-100 dark:bg-neutral-700";

const buttonBase = "px-5 py-2 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed";

export default function ConvertText() {
    const t = useTranslations("ConvertText");
    const initialState: ConvertTextState = { status: null, messageKey: null, converted_text: '' };
    const [state, formAction, isPending] = useActionState(convertTextAction, initialState);
    const [direction, setDirection] = useState<ConversionDirection>("s2t");
    const [inputText, setInputText] = useState("");
    // 結果由本元件持有（而非直接讀 state.converted_text）：換方向 / 交換 / 清除都要能讓它失效，
    // 而 useActionState 的 state 只能由 action 改。
    const [result, setResult] = useState("");
    const { toast, showToast } = useToast();
    const formRef = useRef<HTMLFormElement>(null);
    const radioRefs = useRef<Array<HTMLButtonElement | null>>([]);

    const byteLength = useMemo(() => utf8ByteLength(inputText), [inputText]);
    const overLimit = byteLength > CONVERT_TEXT_MAX_BYTES;
    const canSubmit = !isPending && !overLimit && inputText.trim().length > 0;

    const changeDirection = (value: ConversionDirection) => {
        if (value === direction) return;
        setDirection(value);
        // 方向換了，畫面上那份結果就是「上一個方向」的產物。留著會讓標籤寫繁→簡、
        // 內容卻是簡→繁的字，比清空更誤導。
        setResult("");
    };

    const handleDirectionKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
        const delta =
            e.key === "ArrowRight" || e.key === "ArrowDown" ? 1
                : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1
                    : 0;
        if (delta === 0) return;
        e.preventDefault();
        const next = (index + delta + DIRECTIONS.length) % DIRECTIONS.length;
        changeDirection(DIRECTIONS[next].value);
        radioRefs.current[next]?.focus();
    };

    // 反轉方向並把結果丟回輸入 = 一鍵「再轉回去」
    const handleSwap = () => {
        setDirection(prev => (prev === "s2t" ? "t2s" : "s2t"));
        if (result) setInputText(result);
        setResult("");
    };

    const handleClear = () => {
        setInputText("");
        setResult("");
    };

    // 長文貼上後手不用離開鍵盤（textarea 裡的 Enter 是換行，所以要加修飾鍵）
    const handleInputKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key !== "Enter" || !(e.metaKey || e.ctrlKey)) return;
        e.preventDefault();
        if (!canSubmit) return;
        formRef.current?.requestSubmit();
    };

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(result);
            showToast("success", t("copySuccess"));
        } catch {
            showToast("error", t("copyFail"));
        }
    };

    // action state 變更時同步結果與提示（adjust-state-during-render 模式，取代 useEffect）
    const [prevState, setPrevState] = useState(state);
    if (prevState !== state) {
        setPrevState(state);
        // 只有成功才覆寫結果。失敗時什麼都不做 —— 畫面上那份留著（＝item「失敗不該抹掉上次結果」），
        // 而且不會把剛被清除 / 交換掉的舊結果從 state.converted_text 撈回來。
        if (state.status === 'success') setResult(state.converted_text);
        if (state.status && state.messageKey) {
            showToast(state.status, t(state.messageKey, { maxKb: CONVERT_TEXT_MAX_KB }));
        }
    }

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t("title")} />

            <form ref={formRef} action={formAction} className="flex flex-col gap-3">
                <input type="hidden" name="direction" value={direction} />

                <div className="flex flex-wrap items-center gap-2">
                    {/* 互斥單選 => radiogroup + roving tabindex，不是一排 aria-pressed 的 toggle */}
                    <div role="radiogroup" aria-label={t("directionLabel")} className="flex flex-wrap gap-2">
                        {DIRECTIONS.map(({ value, labelKey }, index) => (
                            <button
                                key={value}
                                ref={el => { radioRefs.current[index] = el; }}
                                type="button"
                                role="radio"
                                aria-checked={direction === value}
                                tabIndex={direction === value ? 0 : -1}
                                onClick={() => changeDirection(value)}
                                onKeyDown={e => handleDirectionKeyDown(e, index)}
                                className={`px-4 py-1.5 rounded-md border text-sm font-medium transition-colors ${direction === value
                                    ? "bg-primary-500 text-white border-primary-500"
                                    : `${neutralBorder} hover:border-primary-400`}`}
                            >
                                {t(labelKey)}
                            </button>
                        ))}
                    </div>
                    <button
                        type="button"
                        onClick={handleSwap}
                        disabled={isPending}
                        title={t("swap")}
                        aria-label={t("swap")}
                        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md border text-sm transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${neutralBorder} hover:bg-neutral-100 dark:hover:bg-neutral-800`}
                    >
                        <ArrowLeftRight className="w-4 h-4" />
                        <span className="hidden sm:inline">{t("swap")}</span>
                    </button>
                </div>

                <label htmlFor="convert-input" className="text-sm text-neutral-600 dark:text-neutral-300">
                    {t("inputLabel")}
                </label>
                <textarea
                    id="convert-input"
                    name="inputText"
                    rows={5}
                    value={inputText}
                    onChange={e => setInputText(e.target.value)}
                    onKeyDown={handleInputKeyDown}
                    readOnly={isPending}
                    aria-describedby="convert-counter"
                    aria-invalid={overLimit || undefined}
                    placeholder={direction === "s2t" ? t("placeholderS2t") : t("placeholderT2s")}
                    className={`${textareaBase} ${inputBg} ${overLimit ? "border-red-500" : neutralBorder} read-only:opacity-60`}
                />
                <p
                    id="convert-counter"
                    aria-live="polite"
                    className={`text-xs ${overLimit ? "text-red-600 dark:text-red-400" : "text-neutral-500 dark:text-neutral-400"}`}
                >
                    {t("counter", {
                        chars: inputText.length,
                        kb: (byteLength / 1024).toFixed(1),
                        maxKb: CONVERT_TEXT_MAX_KB,
                    })}
                    {overLimit && <> — {t("tooLong", { maxKb: CONVERT_TEXT_MAX_KB })}</>}
                </p>

                <div className="flex flex-wrap items-center gap-3">
                    <button
                        type="submit"
                        className={`${buttonBase} bg-primary-500 text-white hover:bg-primary-600`}
                        disabled={!canSubmit}
                    >
                        {isPending ? t("converting") : t("convert")}
                    </button>
                    <button
                        type="button"
                        onClick={handleClear}
                        disabled={isPending || (!inputText && !result)}
                        className={`${buttonBase} border ${neutralBorder} hover:bg-neutral-100 dark:hover:bg-neutral-800`}
                    >
                        {t("clear")}
                    </button>
                    <span className="text-xs text-neutral-500 dark:text-neutral-400">{t("shortcutHint")}</span>
                </div>
            </form>

            <div className="flex flex-col gap-3">
                <h2 id="convert-result-heading" className="text-lg font-semibold text-neutral-800 dark:text-neutral-100">
                    {t("resultLabel")}
                </h2>
                <textarea
                    rows={5}
                    readOnly
                    aria-labelledby="convert-result-heading"
                    value={result}
                    className={`${textareaBase} ${resultBg} ${neutralBorder}`}
                />
                <button
                    type="button"
                    onClick={handleCopy}
                    disabled={!result.trim()}
                    className={`self-start ${buttonBase} border ${neutralBorder} hover:bg-neutral-100 dark:hover:bg-neutral-800`}
                >
                    {t("copy")}
                </button>
            </div>

            <Toast toast={toast} />
        </PageShell>
    );
}

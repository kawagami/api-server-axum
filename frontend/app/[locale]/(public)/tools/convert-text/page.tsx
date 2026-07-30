"use client";

import { useState, useActionState } from "react";
import { useTranslations } from "next-intl";
import { convertTextAction, type ConvertTextState } from "@/app/[locale]/(public)/tools/convert-text/actions";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import Toast, { useToast } from "@/components/toast";

const DIRECTIONS = [
    { value: "s2t", labelKey: "s2t" },
    { value: "t2s", labelKey: "t2s" },
] as const;

const inputClass =
    "w-full p-3 text-base border border-neutral-300 rounded-md bg-white dark:bg-neutral-800 dark:border-neutral-600";

export default function ConvertText() {
    const t = useTranslations("ConvertText");
    const initialState: ConvertTextState = { status: null, messageKey: null, converted_text: '' };
    const [state, formAction, isPending] = useActionState(convertTextAction, initialState);
    const [direction, setDirection] = useState<"s2t" | "t2s">("s2t");
    const { toast, showToast } = useToast();

    const handleCopy = async () => {
        if (!state.converted_text.trim()) {
            showToast("error", t("copyEmpty"));
            return;
        }
        try {
            await navigator.clipboard.writeText(state.converted_text);
            showToast("success", t("copySuccess"));
        } catch {
            showToast("error", t("copyFail"));
        }
    };

    // action state 變更時同步提示（adjust-state-during-render 模式，取代 useEffect）
    const [prevState, setPrevState] = useState(state);
    if (prevState !== state) {
        setPrevState(state);
        if (state.status && state.messageKey) {
            showToast(state.status, t(state.messageKey));
        }
    }

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t("title")} />

            <form action={formAction} className="flex flex-col gap-3">
                <input type="hidden" name="direction" value={direction} />
                <div className="flex flex-wrap gap-2">
                    {DIRECTIONS.map(({ value, labelKey }) => (
                        <button
                            key={value}
                            type="button"
                            onClick={() => setDirection(value)}
                            aria-pressed={direction === value}
                            className={`px-4 py-1.5 rounded-md border text-sm font-medium transition-colors ${direction === value
                                ? "bg-primary-500 text-white border-primary-500"
                                : "border-neutral-300 hover:border-primary-400 dark:border-neutral-600"}`}
                        >
                            {t(labelKey)}
                        </button>
                    ))}
                </div>
                <textarea
                    name="inputText"
                    rows={5}
                    aria-label={t("title")}
                    placeholder={direction === "s2t" ? t("placeholderS2t") : t("placeholderT2s")}
                    className={inputClass}
                />
                <button
                    type="submit"
                    className="self-start px-5 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 transition-colors"
                    disabled={isPending}
                >
                    {isPending ? t("converting") : t("convert")}
                </button>
            </form>

            <div className="flex flex-col gap-3">
                <h2 className="text-lg font-semibold text-neutral-800 dark:text-neutral-100">{t("resultLabel")}</h2>
                <textarea
                    rows={5}
                    readOnly
                    aria-label={t("resultLabel")}
                    value={state.converted_text}
                    className={`${inputClass} bg-neutral-100 dark:bg-neutral-700`}
                />
                <button
                    onClick={handleCopy}
                    className="self-start px-5 py-2 border border-neutral-300 dark:border-neutral-600 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                >
                    {t("copy")}
                </button>
            </div>

            <Toast toast={toast} />
        </PageShell>
    );
}

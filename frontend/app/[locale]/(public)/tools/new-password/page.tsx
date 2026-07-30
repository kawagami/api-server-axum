"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { Copy, Loader2 } from "lucide-react";
import { getNewPassword } from "@/api/tools";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import Toast, { useToast } from "@/components/toast";

const numberInputClass =
    "w-24 px-3 py-2 rounded-md border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100";

export default function NewPasswordPage() {
    const t = useTranslations("NewPassword");
    const [count, setCount] = useState(5);
    const [length, setLength] = useState(12);
    const [newPasswords, setNewPasswords] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState(false);
    const { toast, showToast } = useToast();

    const fetchNewPasswords = async () => {
        setIsLoading(true);
        setError(false);
        try {
            setNewPasswords(await getNewPassword(count, length));
        } catch {
            // 失敗訊息放結果區，不再假裝成一組密碼塞進清單裡
            setNewPasswords([]);
            setError(true);
        } finally {
            setIsLoading(false);
        }
    };

    const handleCopy = async (password: string) => {
        try {
            await navigator.clipboard.writeText(password);
            showToast("success", t("copySuccess"));
        } catch {
            showToast("error", t("copyFail"));
        }
    };

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t("title")} />

            <div className="flex flex-wrap items-end gap-4">
                <div className="flex flex-col gap-1">
                    <label htmlFor="count" className="text-sm text-neutral-600 dark:text-neutral-300">{t("countLabel")}</label>
                    <input
                        id="count"
                        type="number"
                        min={1}
                        max={50}
                        value={count}
                        onChange={(e) => setCount(Math.min(Math.max(Number(e.target.value), 1), 50))}
                        className={numberInputClass}
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label htmlFor="length" className="text-sm text-neutral-600 dark:text-neutral-300">{t("lengthLabel")}</label>
                    <input
                        id="length"
                        type="number"
                        min={1}
                        max={300}
                        value={length}
                        onChange={(e) => setLength(Math.min(Math.max(Number(e.target.value), 1), 300))}
                        className={numberInputClass}
                    />
                </div>
                <button
                    onClick={fetchNewPasswords}
                    disabled={isLoading}
                    className="flex items-center gap-2 px-4 py-2 bg-primary-500 text-white rounded-md hover:bg-primary-600 disabled:opacity-50 transition-colors"
                >
                    {isLoading && <Loader2 className="w-4 h-4 animate-spin" />}
                    {isLoading ? t("generating") : t("generate")}
                </button>
            </div>

            <div className="flex flex-col gap-3">
                <h2 className="text-lg font-semibold text-neutral-800 dark:text-neutral-100">{t("resultLabel")}</h2>
                {isLoading ? (
                    <div className="flex flex-col gap-2">
                        {Array.from({ length: 3 }).map((_, i) => (
                            <div key={i} className="h-9 rounded-md bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                        ))}
                    </div>
                ) : error ? (
                    <p className="text-sm text-red-500 dark:text-red-400">{t("error")}</p>
                ) : newPasswords.length > 0 ? (
                    <ul className="flex flex-col gap-2">
                        {newPasswords.map((password, index) => (
                            <li
                                key={index}
                                className="flex items-center justify-between gap-3 rounded-md bg-white dark:bg-neutral-800 px-3 py-2 shadow-sm"
                            >
                                <span className="font-mono text-sm break-all text-neutral-800 dark:text-neutral-100">{password}</span>
                                <button
                                    onClick={() => handleCopy(password)}
                                    className="shrink-0 flex items-center gap-1 px-2 py-1 text-sm rounded border border-neutral-200 dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
                                >
                                    <Copy size={14} />
                                    {t("copy")}
                                </button>
                            </li>
                        ))}
                    </ul>
                ) : (
                    <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("empty")}</p>
                )}
            </div>

            <Toast toast={toast} />
        </PageShell>
    );
}

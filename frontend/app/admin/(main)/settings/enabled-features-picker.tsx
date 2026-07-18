"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Power, PowerOff } from "lucide-react";
import { updateEnabledFeatures } from "./actions";
import { BACKEND_FEATURES } from "@/libs/enabled-features";

// instance 功能開關管理：存 app_settings 的 enabled_features（"all" 或 JSON 字串陣列）。
// 與首頁卡片（home_features，純展示）不同，這裡關掉 = 後端 API 回 404、排程停跑、全站導航隱藏。
export default function EnabledFeaturesPicker({ initialValue }: { initialValue: string }) {
    const router = useRouter();
    const parse = (value: string): string[] | null => {
        if (value === "all") return null;
        try {
            const parsed = JSON.parse(value);
            return Array.isArray(parsed) ? parsed.filter((k): k is string => typeof k === "string") : null;
        } catch {
            return null;
        }
    };

    const initial = parse(initialValue);
    const [allMode, setAllMode] = useState(initial === null);
    const [selected, setSelected] = useState<string[]>(
        initial ?? BACKEND_FEATURES.map((f) => f.key)
    );
    const [saved, setSaved] = useState(initialValue);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const currentValue = allMode ? "all" : JSON.stringify(selected);
    const dirty = currentValue !== saved;

    function toggle(key: string) {
        setSelected((prev) => {
            if (prev.includes(key)) {
                let next = prev.filter((k) => k !== key);
                // portfolio 依賴 stocks（市價/股名靠股票排程餵）：關 stocks 連帶關 portfolio
                if (key === "stocks") next = next.filter((k) => k !== "portfolio");
                return next;
            }
            const next = [...prev, key];
            if (key === "portfolio" && !next.includes("stocks")) next.push("stocks");
            return next;
        });
    }

    async function save() {
        if (saving || !dirty) return;
        setError(null);
        setSaving(true);
        try {
            await updateEnabledFeatures(currentValue);
            setSaved(currentValue);
            router.refresh();
        } catch (err) {
            setError((err as Error).message);
        } finally {
            setSaving(false);
        }
    }

    return (
        <div className="bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 mb-6">
            <p className="text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-3">
                功能開關
                <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">enabled_features</span>
            </p>

            <label className="flex items-center gap-2 mb-3 text-sm text-neutral-700 dark:text-neutral-300">
                <input
                    type="checkbox"
                    checked={allMode}
                    disabled={saving}
                    onChange={() => setAllMode((m) => !m)}
                    className="accent-primary-600"
                />
                全部啟用（含未來新增的功能）
            </label>

            {!allMode && (
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                    {BACKEND_FEATURES.map(({ key, label }) => {
                        const on = selected.includes(key);
                        return (
                            <button
                                key={key}
                                onClick={() => toggle(key)}
                                disabled={saving}
                                className={`flex items-center gap-2 px-3 py-2 rounded-lg border text-left text-sm transition-colors
                                    ${on
                                        ? "border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-900/30 text-neutral-800 dark:text-neutral-200"
                                        : "border-neutral-200 dark:border-neutral-700 text-neutral-500 dark:text-neutral-400 opacity-70"
                                    }`}
                            >
                                {on
                                    ? <Power size={14} className="text-primary-600 dark:text-primary-400 shrink-0" />
                                    : <PowerOff size={14} className="shrink-0" />}
                                <span className="flex-1 min-w-0 truncate">
                                    {label}
                                    <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono hidden sm:inline">{key}</span>
                                </span>
                            </button>
                        );
                    })}
                </div>
            )}

            <div className="flex items-center gap-3 mt-4">
                <button
                    onClick={save}
                    disabled={saving || !dirty}
                    className="px-4 py-2 text-sm font-medium bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg transition-colors"
                >
                    {saving ? "儲存中..." : "儲存"}
                </button>
                {dirty && !saving && (
                    <span className="text-xs text-neutral-500 dark:text-neutral-400">有未儲存的變更</span>
                )}
            </div>

            {error && <p className="mt-2 text-xs text-red-500">{error}</p>}
            <p className="mt-2 text-xs text-neutral-400 dark:text-neutral-500">
                關閉的功能：API 回 404、排程停跑、前台/後台導航隱藏（公開頁最多延遲 60 秒）。
                投資組合依賴股票追蹤的排程資料，會連動開關。
            </p>
        </div>
    );
}

"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { ArrowDown, ArrowUp, Eye, EyeOff } from "lucide-react";
import { updateHomeFeatures } from "./actions";
import { HOME_FEATURES } from "@/libs/home-features";

// 首頁功能卡片管理：顯示/隱藏 + 排序，存 app_settings 的 home_features（JSON 字串陣列）
export default function HomeFeaturesPicker({ initialEnabled }: { initialEnabled: string[] }) {
    const router = useRouter();
    const [enabled, setEnabled] = useState<string[]>(initialEnabled);
    const [saved, setSaved] = useState<string[]>(initialEnabled);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const dirty = enabled.join(",") !== saved.join(",");
    const disabled = HOME_FEATURES.filter((f) => !enabled.includes(f.key));

    function toggle(key: string) {
        setEnabled((prev) =>
            prev.includes(key) ? prev.filter((k) => k !== key) : [...prev, key]
        );
    }

    function move(key: string, dir: -1 | 1) {
        setEnabled((prev) => {
            const index = prev.indexOf(key);
            const target = index + dir;
            if (index < 0 || target < 0 || target >= prev.length) return prev;
            const next = [...prev];
            [next[index], next[target]] = [next[target], next[index]];
            return next;
        });
    }

    async function save() {
        if (saving || !dirty) return;
        setError(null);
        setSaving(true);
        try {
            await updateHomeFeatures(enabled);
            setSaved(enabled);
            router.refresh();
        } catch (err) {
            setError((err as Error).message);
        } finally {
            setSaving(false);
        }
    }

    const rowClass =
        "flex items-center gap-2 px-3 py-2 rounded-lg border border-neutral-200 dark:border-neutral-700";

    return (
        <div className="bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 mb-6">
            <p className="text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-3">
                首頁功能卡片
                <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">home_features</span>
            </p>

            <div className="space-y-2">
                {enabled.map((key, index) => {
                    const feature = HOME_FEATURES.find((f) => f.key === key);
                    if (!feature) return null;
                    const Icon = feature.icon;
                    return (
                        <div key={key} className={rowClass}>
                            <Icon size={16} className="text-primary-600 dark:text-primary-400 shrink-0" />
                            <span className="flex-1 text-sm text-neutral-800 dark:text-neutral-200">
                                {feature.label}
                                <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">{key}</span>
                            </span>
                            <button
                                onClick={() => move(key, -1)}
                                disabled={saving || index === 0}
                                className="p-1.5 rounded text-neutral-500 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-700 disabled:opacity-30 transition-colors"
                                aria-label="上移"
                            >
                                <ArrowUp size={14} />
                            </button>
                            <button
                                onClick={() => move(key, 1)}
                                disabled={saving || index === enabled.length - 1}
                                className="p-1.5 rounded text-neutral-500 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-700 disabled:opacity-30 transition-colors"
                                aria-label="下移"
                            >
                                <ArrowDown size={14} />
                            </button>
                            <button
                                onClick={() => toggle(key)}
                                disabled={saving}
                                className="flex items-center gap-1 px-2 py-1.5 rounded text-xs text-primary-700 dark:text-primary-300 hover:bg-primary-50 dark:hover:bg-primary-900/30 transition-colors"
                            >
                                <Eye size={14} />
                                顯示中
                            </button>
                        </div>
                    );
                })}

                {disabled.map((feature) => {
                    const Icon = feature.icon;
                    return (
                        <div key={feature.key} className={`${rowClass} opacity-60`}>
                            <Icon size={16} className="text-neutral-400 dark:text-neutral-500 shrink-0" />
                            <span className="flex-1 text-sm text-neutral-500 dark:text-neutral-400">
                                {feature.label}
                                <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">{feature.key}</span>
                            </span>
                            <button
                                onClick={() => toggle(feature.key)}
                                disabled={saving}
                                className="flex items-center gap-1 px-2 py-1.5 rounded text-xs text-neutral-500 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
                            >
                                <EyeOff size={14} />
                                已隱藏
                            </button>
                        </div>
                    );
                })}
            </div>

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
                全站設定，所有訪客生效（公開頁最多延遲 60 秒）；全部隱藏時首頁只剩簡介區塊
            </p>
        </div>
    );
}

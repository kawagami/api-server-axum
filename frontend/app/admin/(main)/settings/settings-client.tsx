"use client";

import { useState } from "react";
import { Check, Eye, EyeOff } from "lucide-react";
import { updateSetting } from "./actions";
import type { Setting, SettingsResponse } from "@/types";
import { FIELD_CONFIGS, CATEGORY_LABELS } from "./field-config";

const inputClass = "px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-600 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500";

export default function SettingsClient({ initialSettings }: { initialSettings: SettingsResponse }) {
    const initialValues = Object.fromEntries(
        Object.values(initialSettings).flat().map(s => [s.key, s.value])
    );
    // description / category 不會在 client 端變動,結構直接讀 initialSettings;只有 value 進 state
    const [values, setValues] = useState<Record<string, string>>(initialValues);
    const [drafts, setDrafts] = useState<Record<string, string>>(initialValues);
    const [saving, setSaving] = useState<Record<string, boolean>>({});
    const [saved, setSaved] = useState<Record<string, boolean>>({});
    const [errors, setErrors] = useState<Record<string, string>>({});
    const [revealed, setRevealed] = useState<Record<string, boolean>>({});

    const handleSave = async (key: string) => {
        const config = FIELD_CONFIGS[key];
        const draft = drafts[key] ?? "";
        if (config?.kind === "number") {
            if (!/^\d+$/.test(draft)) {
                setErrors(prev => ({ ...prev, [key]: "必須是整數" }));
                return;
            }
            if (config.min !== undefined && Number(draft) < config.min) {
                setErrors(prev => ({ ...prev, [key]: `最小值為 ${config.min}` }));
                return;
            }
        }
        setSaving(prev => ({ ...prev, [key]: true }));
        setErrors(prev => ({ ...prev, [key]: "" }));
        try {
            const updated = await updateSetting(key, draft);
            setValues(prev => ({ ...prev, [key]: updated.value }));
            setDrafts(prev => ({ ...prev, [key]: updated.value }));
            setSaved(prev => ({ ...prev, [key]: true }));
            setTimeout(() => setSaved(prev => ({ ...prev, [key]: false })), 2000);
        } catch (err) {
            setErrors(prev => ({ ...prev, [key]: (err as Error).message }));
        } finally {
            setSaving(prev => ({ ...prev, [key]: false }));
        }
    };

    const renderInput = (setting: Setting) => {
        const config = FIELD_CONFIGS[setting.key];
        const draft = drafts[setting.key] ?? "";
        const onChange = (value: string) =>
            setDrafts(prev => ({ ...prev, [setting.key]: value }));

        if (config?.kind === "enum") {
            return (
                <select
                    value={draft}
                    onChange={e => onChange(e.target.value)}
                    className={`${inputClass} flex-1 min-w-0`}
                >
                    {/* 現值不在選項中時保留顯示,避免 select 靜默換值 */}
                    {!config.options.includes(draft) && (
                        <option value={draft}>{draft || "(空值)"}</option>
                    )}
                    {config.options.map(opt => (
                        <option key={opt} value={opt}>{opt}</option>
                    ))}
                </select>
            );
        }

        if (config?.kind === "number") {
            return (
                <input
                    type="number"
                    min={config.min}
                    value={draft}
                    onChange={e => onChange(e.target.value)}
                    className={`${inputClass} flex-1 min-w-0`}
                    placeholder={setting.description}
                />
            );
        }

        if (config?.kind === "secret") {
            const shown = revealed[setting.key];
            return (
                <div className="relative flex-1 min-w-0">
                    <input
                        type={shown ? "text" : "password"}
                        autoComplete="new-password"
                        value={draft}
                        onChange={e => onChange(e.target.value)}
                        className={`${inputClass} w-full pr-10`}
                        placeholder={setting.description}
                    />
                    <button
                        type="button"
                        onClick={() => setRevealed(prev => ({ ...prev, [setting.key]: !shown }))}
                        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 transition-colors"
                        aria-label={shown ? "隱藏" : "顯示"}
                    >
                        {shown ? <EyeOff size={16} /> : <Eye size={16} />}
                    </button>
                </div>
            );
        }

        return (
            <input
                type="text"
                value={draft}
                onChange={e => onChange(e.target.value)}
                className={`${inputClass} flex-1 min-w-0`}
                placeholder={setting.description}
            />
        );
    };

    const renderSetting = (setting: Setting) => {
        const dirty = (drafts[setting.key] ?? "") !== (values[setting.key] ?? "");
        return (
            <div key={setting.key} className="bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4">
                <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
                    {setting.description}
                    <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">{setting.key}</span>
                </label>
                <div className="flex gap-2 mt-2">
                    {renderInput(setting)}
                    <button
                        onClick={() => handleSave(setting.key)}
                        disabled={saving[setting.key] || !dirty}
                        className="shrink-0 px-4 py-2 text-sm font-medium bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg transition-colors"
                    >
                        {saving[setting.key] ? "儲存中..." : "儲存"}
                    </button>
                </div>
                {errors[setting.key] && (
                    <p className="mt-1 text-xs text-red-500">{errors[setting.key]}</p>
                )}
                {saved[setting.key] && (
                    <p className="mt-1 text-xs text-green-600 dark:text-green-400 flex items-center gap-1">
                        <Check size={12} /> 已儲存
                    </p>
                )}
            </div>
        );
    };

    return (
        <div className="space-y-8">
            {Object.entries(initialSettings).map(([category, items]) => (
                <section key={category}>
                    <h2 className="flex items-baseline gap-2 text-sm font-semibold text-neutral-900 dark:text-white mb-3">
                        {CATEGORY_LABELS[category] ?? category}
                        <span className="text-xs font-normal text-neutral-400 dark:text-neutral-500 font-mono">{category}</span>
                    </h2>
                    <div className="space-y-3">{items.map(renderSetting)}</div>
                </section>
            ))}
        </div>
    );
}

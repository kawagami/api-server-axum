"use client";

import { useState } from "react";
import { updateSetting } from "./actions";
import type { Setting, SettingsResponse } from "@/types";

export default function SettingsClient({ initialSettings }: { initialSettings: SettingsResponse }) {
    const categories = Object.keys(initialSettings);
    const [activeTab, setActiveTab] = useState(categories[0] ?? "");
    const [settings, setSettings] = useState<SettingsResponse>(initialSettings);
    const [drafts, setDrafts] = useState<Record<string, string>>(
        Object.fromEntries(
            Object.values(initialSettings).flat().map(s => [s.key, s.value])
        )
    );
    const [saving, setSaving] = useState<Record<string, boolean>>({});
    const [errors, setErrors] = useState<Record<string, string>>({});

    const handleSave = async (key: string) => {
        setSaving(prev => ({ ...prev, [key]: true }));
        setErrors(prev => ({ ...prev, [key]: "" }));
        try {
            const updated = await updateSetting(key, drafts[key]);
            setSettings(prev => ({
                ...prev,
                [updated.category]: prev[updated.category].map(s => s.key === key ? updated : s),
            }));
            setDrafts(prev => ({ ...prev, [key]: updated.value }));
        } catch (err) {
            setErrors(prev => ({ ...prev, [key]: (err as Error).message }));
        } finally {
            setSaving(prev => ({ ...prev, [key]: false }));
        }
    };

    const renderSetting = (setting: Setting) => (
        <div key={setting.key} className="bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4">
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
                {setting.description}
                <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">{setting.key}</span>
            </label>
            <div className="flex gap-2 mt-2">
                <input
                    type="text"
                    value={drafts[setting.key] ?? ""}
                    onChange={e => setDrafts(prev => ({ ...prev, [setting.key]: e.target.value }))}
                    className="flex-1 px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-600 rounded-lg bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                    placeholder={setting.description}
                />
                <button
                    onClick={() => handleSave(setting.key)}
                    disabled={saving[setting.key]}
                    className="px-4 py-2 text-sm font-medium bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg transition-colors"
                >
                    {saving[setting.key] ? "儲存中..." : "儲存"}
                </button>
            </div>
            {errors[setting.key] && (
                <p className="mt-1 text-xs text-red-500">{errors[setting.key]}</p>
            )}
        </div>
    );

    return (
        <div>
            <div className="flex gap-1 border-b border-neutral-200 dark:border-neutral-700 mb-6">
                {categories.map(cat => (
                    <button
                        key={cat}
                        onClick={() => setActiveTab(cat)}
                        className={`px-4 py-2 text-sm font-medium capitalize transition-colors border-b-2 -mb-px
                            ${activeTab === cat
                                ? "border-primary-600 text-primary-600 dark:text-primary-400 dark:border-primary-400"
                                : "border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200"
                            }`}
                    >
                        {cat}
                    </button>
                ))}
            </div>
            <div className="space-y-3">
                {(settings[activeTab] ?? []).map(renderSetting)}
            </div>
        </div>
    );
}

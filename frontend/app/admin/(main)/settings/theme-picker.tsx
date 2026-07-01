"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Trees, Waves, CloudSun, Sunset, Flower2, Grape, Contrast, Repeat } from "lucide-react";
import { updateSiteTheme, updateThemeRotation } from "./actions";
import {
    SITE_THEMES,
    SITE_THEME_LABELS,
    AUTO_THEME,
    applySiteThemeAttr,
    resolveActiveTheme,
    type SiteTheme,
    type SiteThemeSetting,
    type ThemeRotation,
} from "@/libs/site-theme";

const THEME_ICONS: Record<SiteTheme, typeof Trees> = {
    forest: Trees,
    ocean: Waves,
    sky: CloudSun,
    sunset: Sunset,
    sakura: Flower2,
    grape: Grape,
    mono: Contrast,
};

const WEEKDAYS = ['週日', '週一', '週二', '週三', '週四', '週五', '週六'];

export default function ThemePicker({
    initialSetting,
    initialRotation,
}: {
    initialSetting: SiteThemeSetting;
    initialRotation: ThemeRotation;
}) {
    const router = useRouter();
    const [setting, setSetting] = useState<SiteThemeSetting>(initialSetting);
    const [rotation, setRotation] = useState<ThemeRotation>(initialRotation);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // 樂觀套用 data-theme：具體主題直接套，auto 套今天輪到的那套
    function previewAttr(next: SiteThemeSetting, rot: ThemeRotation) {
        applySiteThemeAttr(next === AUTO_THEME ? resolveActiveTheme(AUTO_THEME, rot) : next);
    }

    async function applySetting(next: SiteThemeSetting) {
        if (saving || next === setting) return;
        const prev = setting;
        setError(null);
        setSetting(next);
        previewAttr(next, rotation);
        setSaving(true);
        try {
            await updateSiteTheme(next);
            router.refresh();
        } catch (err) {
            setSetting(prev);
            previewAttr(prev, rotation);
            setError((err as Error).message);
        } finally {
            setSaving(false);
        }
    }

    async function changeRotationDay(day: number, value: SiteTheme) {
        if (saving) return;
        const prev = rotation;
        const next = { ...rotation, [String(day)]: value };
        setError(null);
        setRotation(next);
        if (setting === AUTO_THEME) previewAttr(AUTO_THEME, next);
        setSaving(true);
        try {
            await updateThemeRotation(next);
            router.refresh();
        } catch (err) {
            setRotation(prev);
            if (setting === AUTO_THEME) previewAttr(AUTO_THEME, prev);
            setError((err as Error).message);
        } finally {
            setSaving(false);
        }
    }

    const autoActive = setting === AUTO_THEME;

    return (
        <div className="bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 mb-6">
            <p className="text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-3">
                網站風格
                <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500 font-mono">site_theme</span>
            </p>
            <div className="flex flex-wrap gap-3">
                {SITE_THEMES.map((name) => {
                    const Icon = THEME_ICONS[name];
                    const active = setting === name;
                    return (
                        <button
                            key={name}
                            onClick={() => applySetting(name)}
                            disabled={saving}
                            className={`flex items-center gap-2 px-4 py-2 rounded-lg border-2 text-sm font-medium transition-colors disabled:opacity-60 ${
                                active
                                    ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300'
                                    : 'border-neutral-200 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400 hover:border-primary-300'
                            }`}
                        >
                            <Icon size={16} />
                            {SITE_THEME_LABELS[name]}
                        </button>
                    );
                })}
                {/* auto：每日輪播 */}
                <button
                    onClick={() => applySetting(AUTO_THEME)}
                    disabled={saving}
                    className={`flex items-center gap-2 px-4 py-2 rounded-lg border-2 text-sm font-medium transition-colors disabled:opacity-60 ${
                        autoActive
                            ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300'
                            : 'border-neutral-200 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400 hover:border-primary-300'
                    }`}
                >
                    <Repeat size={16} />
                    每日輪播
                </button>
            </div>

            {/* 輪播對應表：只有選 auto 才顯示 */}
            {autoActive && (
                <div className="mt-4 border-t border-neutral-200 dark:border-neutral-700 pt-4">
                    <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-3">
                        每天套用對應主題（時區 Asia/Taipei）
                        <span className="ml-2 font-mono">theme_rotation</span>
                    </p>
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                        {WEEKDAYS.map((label, day) => (
                            <label key={day} className="flex flex-col gap-1 text-sm">
                                <span className="text-neutral-600 dark:text-neutral-400">{label}</span>
                                <select
                                    value={rotation[String(day)] ?? 'forest'}
                                    onChange={(e) => changeRotationDay(day, e.target.value as SiteTheme)}
                                    disabled={saving}
                                    className="rounded-md border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900 px-2 py-1.5 text-neutral-800 dark:text-neutral-200 disabled:opacity-60"
                                >
                                    {SITE_THEMES.map((name) => (
                                        <option key={name} value={name}>
                                            {SITE_THEME_LABELS[name]}
                                        </option>
                                    ))}
                                </select>
                            </label>
                        ))}
                    </div>
                </div>
            )}

            {error && <p className="mt-2 text-xs text-red-500">{error}</p>}
            <p className="mt-2 text-xs text-neutral-400 dark:text-neutral-500">
                全站設定，所有訪客生效（公開頁最多延遲 60 秒）
            </p>
        </div>
    );
}

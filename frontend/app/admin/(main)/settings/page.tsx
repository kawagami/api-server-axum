import { getSettings } from "./actions";
import SettingsClient from "./settings-client";
import ThemePicker from "./theme-picker";
import HomeFeaturesPicker from "./home-features-picker";
import { resolveSiteThemeSetting, normalizeRotation } from "@/libs/site-theme";
import { resolveHomeFeatures } from "@/libs/home-features";
import { requirePermission } from "@/libs/admin-permissions";
import type { SettingsResponse } from "@/types";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Settings",
    description: "Admin settings",
};

export default async function SettingsPage() {
    await requirePermission("setting:read");
    const settings = await getSettings();
    const flat = Object.values(settings).flat();

    // site_theme / theme_rotation / home_features 由專屬 UI 管理，從通用設定表單中拿掉避免兩處改同一 key
    const siteTheme = resolveSiteThemeSetting(flat.find(s => s.key === 'site_theme')?.value);
    const rotation = normalizeRotation(flat.find(s => s.key === 'theme_rotation')?.value);
    const homeFeatureKeys = resolveHomeFeatures(flat.find(s => s.key === 'home_features')?.value).map(f => f.key);
    const MANAGED_KEYS = ['site_theme', 'theme_rotation', 'home_features'];
    const restSettings: SettingsResponse = Object.fromEntries(
        Object.entries(settings)
            .map(([category, items]) => [category, items.filter(s => !MANAGED_KEYS.includes(s.key))] as const)
            .filter(([, items]) => items.length > 0)
    );

    return (
        <div className="w-full max-w-2xl">
            <h1 className="text-xl font-semibold text-neutral-900 dark:text-white mb-6">設定</h1>
            <ThemePicker initialSetting={siteTheme} initialRotation={rotation} />
            <HomeFeaturesPicker initialEnabled={homeFeatureKeys} />
            <SettingsClient initialSettings={restSettings} />
        </div>
    );
}

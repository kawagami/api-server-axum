import { getSettings } from "../settings/actions";
import EnabledFeaturesPicker from "./enabled-features-picker";
import { requirePermission } from "@/libs/admin-permissions";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "平台設定",
    description: "Platform reserved settings",
};

// 平台保留設定頁：只有 platform:read 看得到（sidebar 同權限過濾）。
// 商家 instance 的管理員拿 setting:* 管日常設定，這頁與保留 key 對他們不存在。
export default async function PlatformSettingsPage() {
    await requirePermission("platform:read");
    const settings = await getSettings();
    const flat = Object.values(settings).flat();
    const enabledFeaturesValue = flat.find(s => s.key === 'enabled_features')?.value ?? 'all';

    return (
        <div className="w-full max-w-2xl">
            <h1 className="text-xl font-semibold text-neutral-900 dark:text-white mb-2">平台設定</h1>
            <p className="text-sm text-neutral-500 dark:text-neutral-400 mb-6">
                平台保留設定，修改需要 platform:update 權限，一般設定權限（setting:*）碰不到這裡的項目。
            </p>
            <EnabledFeaturesPicker initialValue={enabledFeaturesValue} />
        </div>
    );
}

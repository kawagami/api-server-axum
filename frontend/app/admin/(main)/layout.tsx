import { cookies } from "next/headers";
import AdminSidebar from "@/components/admin/admin-sidebar";
import AdminBreadcrumb from "@/components/admin/admin-breadcrumb";
import TokenRefreshInit from "@/components/admin/token-refresh-init";
import { getCurrentAdmin } from "@/libs/admin-permissions";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures } from "@/libs/enabled-features";
import { resolveDefaultColorMode, type UserColorMode } from "@/libs/color-mode";

export default async function AdminMainLayout({ children }: { children: React.ReactNode }) {
    const [admin, publicSettings, cookieStore] = await Promise.all([
        getCurrentAdmin(),
        getPublicSettings(),
        cookies(),
    ]);
    const enabledFeatures = resolveEnabledFeatures(publicSettings.enabled_features);

    // 深淺色切換：與前台共用同一個 theme cookie，優先序也一致（cookie ＞ 站台預設 ＞ 系統）
    const themeCookie = cookieStore.get('theme')?.value;
    const colorMode: UserColorMode =
        themeCookie === 'dark' ? 'dark' : themeCookie === 'light' ? 'light' : 'auto';
    const defaultMode = resolveDefaultColorMode(publicSettings.default_color_mode);
    const defaultIsDark = defaultMode === 'system' ? null : defaultMode === 'dark';

    return (
        <div className="flex w-full h-screen">
            <TokenRefreshInit />
            <AdminSidebar
                admin={{ name: admin.name, isSuperAdmin: admin.is_super_admin }}
                permissions={admin.permissions}
                enabledFeatures={enabledFeatures}
                colorMode={colorMode}
                defaultIsDark={defaultIsDark}
            />
            <div className="flex-1 overflow-auto px-4 pb-4 pt-3 sm:px-6 sm:pb-6 sm:pt-4">
                {/* 內容寬度的單一來源：頁面不要再自己加 max-w / mx-auto，否則切頁時卡片會左右跳。
                    表單、詳情這類窄版頁在自己的最外層用 max-w-2xl（靠左，不置中）。 */}
                <div className="mx-auto w-full max-w-6xl">
                    {/* 手機版：pl-12 讓出固定定位漢堡鈕的空間，與其同列 */}
                    <AdminBreadcrumb className="min-h-8 pl-12 sm:pl-0 mb-3" />
                    {children}
                </div>
            </div>
        </div>
    );
}

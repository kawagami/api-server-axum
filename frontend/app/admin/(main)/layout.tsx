import AdminSidebar from "@/components/admin/admin-sidebar";
import AdminBreadcrumb from "@/components/admin/admin-breadcrumb";
import TokenRefreshInit from "@/components/admin/token-refresh-init";
import { getMyPermissions } from "@/libs/admin-permissions";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures } from "@/libs/enabled-features";

export default async function AdminMainLayout({ children }: { children: React.ReactNode }) {
    const [permissions, publicSettings] = await Promise.all([
        getMyPermissions(),
        getPublicSettings(),
    ]);
    const enabledFeatures = resolveEnabledFeatures(publicSettings.enabled_features);
    return (
        <div className="flex w-full h-screen">
            <TokenRefreshInit />
            <AdminSidebar permissions={permissions} enabledFeatures={enabledFeatures} />
            <div className="flex-1 overflow-auto px-4 pb-4 pt-3 sm:px-6 sm:pb-6 sm:pt-4">
                {/* 手機版：pl-12 讓出固定定位漢堡鈕的空間，與其同列 */}
                <AdminBreadcrumb className="min-h-8 pl-12 sm:pl-0 mb-3" />
                {children}
            </div>
        </div>
    );
}

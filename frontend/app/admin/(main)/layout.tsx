import AdminSidebar from "@/components/admin/admin-sidebar";
import AdminBreadcrumb from "@/components/admin/admin-breadcrumb";
import TokenRefreshInit from "@/components/admin/token-refresh-init";
import { getMyPermissions } from "@/libs/admin-permissions";

export default async function AdminMainLayout({ children }: { children: React.ReactNode }) {
    const permissions = await getMyPermissions();
    return (
        <div className="flex w-full h-screen">
            <TokenRefreshInit />
            <AdminSidebar permissions={permissions} />
            <div className="flex-1 overflow-auto px-4 pb-4 pt-3 sm:px-6 sm:pb-6 sm:pt-4">
                {/* 手機版：pl-12 讓出固定定位漢堡鈕的空間，與其同列 */}
                <AdminBreadcrumb className="min-h-8 pl-12 sm:pl-0 mb-3" />
                {children}
            </div>
        </div>
    );
}

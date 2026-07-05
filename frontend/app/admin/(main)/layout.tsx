import AdminSidebar from "@/components/admin/admin-sidebar";
import TokenRefreshInit from "@/components/admin/token-refresh-init";
import { getMyPermissions } from "@/libs/admin-permissions";

export default async function AdminMainLayout({ children }: { children: React.ReactNode }) {
    const permissions = await getMyPermissions();
    return (
        <div className="flex w-full h-screen">
            <TokenRefreshInit />
            <AdminSidebar permissions={permissions} />
            <div className="flex-1 overflow-auto p-4 sm:p-6">
                {children}
            </div>
        </div>
    );
}

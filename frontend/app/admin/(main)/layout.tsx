import AdminSidebar from "@/components/admin/admin-sidebar";
import TokenRefreshInit from "@/components/admin/token-refresh-init";

export default function AdminMainLayout({ children }: { children: React.ReactNode }) {
    return (
        <div className="flex w-full h-screen">
            <TokenRefreshInit />
            <AdminSidebar />
            <div className="flex-1 overflow-auto p-4 sm:p-6">
                {children}
            </div>
        </div>
    );
}

import { requirePermission } from "@/libs/admin-permissions";

export default async function StocksLayout({ children }: { children: React.ReactNode }) {
    await requirePermission("stock:read");
    return children;
}

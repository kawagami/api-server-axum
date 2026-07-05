import { requirePermission } from "@/libs/admin-permissions";

export default async function MembersLayout({ children }: { children: React.ReactNode }) {
    await requirePermission("member:read");
    return children;
}

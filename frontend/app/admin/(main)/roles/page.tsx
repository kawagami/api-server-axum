import { getRoles } from "@/api/roles";
import { getPermissions } from "@/api/roles";
import RolesManager from "@/components/roles/roles-manager";
import { requirePermission } from "@/libs/admin-permissions";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "角色",
    description: "角色與權限管理",
};

export default async function RolesPage() {
    await requirePermission("role:read");
    const [roles, permissions] = await Promise.all([getRoles(), getPermissions()]);

    return (
        <div className="w-full">
            <RolesManager initialRoles={roles} allPermissions={permissions} />
        </div>
    );
}

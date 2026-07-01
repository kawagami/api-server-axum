import { getRoles } from "@/api/roles";
import { getPermissions } from "@/api/roles";
import RolesManager from "@/components/roles/roles-manager";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "角色",
    description: "角色與權限管理",
};

export default async function RolesPage() {
    const [roles, permissions] = await Promise.all([getRoles(), getPermissions()]);

    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-3 sm:p-6">
            <RolesManager initialRoles={roles} allPermissions={permissions} />
        </div>
    );
}

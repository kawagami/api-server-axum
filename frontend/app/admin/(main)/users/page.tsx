import { getUsers, getUserRoles } from "@/api/users";
import { getRoles } from "@/api/roles";
import { getSettings } from "@/app/admin/(main)/settings/actions";
import UserRolesPanel from "./user-roles-panel";
import CreateUserForm from "./create-user-form";
import type { Metadata } from "next";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";

export const metadata: Metadata = {
    title: "管理員",
    description: "管理員",
};

export default async function Users() {
    const [users, allRoles, settings] = await Promise.all([getUsers(), getRoles(), getSettings()]);

    const usersWithRoles = await Promise.all(
        users.map(async user => ({
            ...user,
            roles: await getUserRoles(user.id),
        }))
    );

    // 新增管理員時預設勾選的角色 — 由 app_settings `new_user_default_roles`（逗號分隔角色名稱）決定
    const defaultRoleNames = (Object.values(settings).flat().find(s => s.key === "new_user_default_roles")?.value ?? "")
        .split(",")
        .map(s => s.trim())
        .filter(Boolean);
    const defaultRoleIds = allRoles.filter(r => defaultRoleNames.includes(r.name)).map(r => r.id);

    return (
        <div>
        <CreateUserForm allRoles={allRoles} defaultRoleIds={defaultRoleIds} />
        <AdminTableContainer>
            <AdminTable>
                <thead>
                    <AdminHeadRow>
                        <AdminTh>ID</AdminTh>
                        <AdminTh>Name</AdminTh>
                        <AdminTh>Email</AdminTh>
                        <AdminTh>Roles</AdminTh>
                    </AdminHeadRow>
                </thead>
                <tbody>
                    {usersWithRoles.map(user => (
                        <AdminRow key={user.id}>
                            <AdminTd className="text-xs">{user.id}</AdminTd>
                            <AdminTd>{user.name}</AdminTd>
                            <AdminTd>{user.email}</AdminTd>
                            <AdminTd>
                                <UserRolesPanel
                                    userId={user.id}
                                    userName={user.name ?? user.email}
                                    initialRoles={user.roles}
                                    allRoles={allRoles}
                                />
                            </AdminTd>
                        </AdminRow>
                    ))}
                </tbody>
            </AdminTable>
        </AdminTableContainer>
        </div>
    );
}

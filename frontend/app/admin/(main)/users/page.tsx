import { getUsers, getUserRoles } from "@/api/users";
import { getRoles } from "@/api/roles";
import UserRolesPanel from "./user-roles-panel";
import type { Metadata } from "next";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";

export const metadata: Metadata = {
    title: "管理員",
    description: "管理員",
};

export default async function Users() {
    const [users, allRoles] = await Promise.all([getUsers(), getRoles()]);

    const usersWithRoles = await Promise.all(
        users.map(async user => ({
            ...user,
            roles: await getUserRoles(user.id),
        }))
    );

    return (
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
    );
}

import { getMembers } from "@/api/members";
import Link from "next/link";
import type { Metadata } from "next";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";

export const metadata: Metadata = {
    title: "Members",
    description: "Members list",
};

export default async function MembersPage() {
    const members = await getMembers();

    return (
        <AdminTableContainer>
            <AdminTable>
                <thead>
                    <AdminHeadRow>
                        <AdminTh>ID</AdminTh>
                        <AdminTh>Name</AdminTh>
                        <AdminTh>Email</AdminTh>
                        <AdminTh>Created At</AdminTh>
                    </AdminHeadRow>
                </thead>
                <tbody>
                    {members.map(member => (
                        <AdminRow key={member.id}>
                            <AdminTd className="text-xs">
                                <Link href={`/admin/members/${member.id}`} className="text-primary-600 dark:text-primary-400 hover:underline">
                                    {member.id}
                                </Link>
                            </AdminTd>
                            <AdminTd>
                                <Link href={`/admin/members/${member.id}`} className="hover:underline">
                                    {member.name}
                                </Link>
                            </AdminTd>
                            <AdminTd>{member.email ?? '—'}</AdminTd>
                            <AdminTd className="text-sm">
                                {new Date(member.created_at).toLocaleString()}
                            </AdminTd>
                        </AdminRow>
                    ))}
                </tbody>
            </AdminTable>
        </AdminTableContainer>
    );
}

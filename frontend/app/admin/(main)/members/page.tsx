import { getMembers } from "@/api/members";
import Link from "next/link";
import type { Metadata } from "next";
import AdminTableContainer from "@/components/admin/admin-table-container";
import PageHeader from "@/components/admin/page-header";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import { formatDateTime } from "@/libs/admin-datetime";

export const metadata: Metadata = {
    title: "會員列表",
    description: "OAuth 註冊的前台會員",
};

export default async function MembersPage() {
    const { data: members, total } = await getMembers();

    return (
        <div className="w-full flex min-h-0 flex-1 flex-col gap-4">
            <PageHeader title="會員列表" description={`共 ${total} 位會員`} />
            <AdminTableContainer stickyHead fill>
                <AdminTable>
                    <thead>
                        <AdminHeadRow>
                            <AdminTh className="hidden sm:table-cell">ID</AdminTh>
                            <AdminTh>名稱</AdminTh>
                            <AdminTh>Email</AdminTh>
                            <AdminTh>註冊時間</AdminTh>
                        </AdminHeadRow>
                    </thead>
                    <tbody>
                        {members.length === 0 ? (
                            <AdminEmptyRow colSpan={4}>目前沒有會員</AdminEmptyRow>
                        ) : (
                            members.map(member => (
                                <AdminRow key={member.id}>
                                    <AdminTd className="text-xs hidden sm:table-cell">
                                        <Link href={`/admin/members/${member.id}`} className="text-primary-600 dark:text-primary-400 hover:underline">
                                            {member.id}
                                        </Link>
                                    </AdminTd>
                                    <AdminTd>
                                        <Link href={`/admin/members/${member.id}`} className="hover:underline">
                                            {member.name}
                                        </Link>
                                    </AdminTd>
                                    <AdminTd className="break-all">{member.email ?? '—'}</AdminTd>
                                    <AdminTd className="text-sm whitespace-nowrap">
                                        {formatDateTime(member.created_at)}
                                    </AdminTd>
                                </AdminRow>
                            ))
                        )}
                    </tbody>
                </AdminTable>
            </AdminTableContainer>
        </div>
    );
}

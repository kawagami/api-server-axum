import { getMember } from "@/api/members";
import Image from "next/image";
import type { Metadata } from "next";
import { AdminTable, AdminTh, AdminTd } from "@/components/admin/table";
import { formatDateTime } from "@/libs/admin-datetime";

export const metadata: Metadata = {
    title: "會員詳情",
    description: "單一會員的基本資料與 OAuth 綁定",
};

export default async function MemberDetailPage({ params }: { params: Promise<{ id: string }> }) {
    const id = Number((await params).id);
    const member = await getMember(id);

    return (
        <div className="w-full max-w-2xl">
            <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg p-4 sm:p-6 space-y-4">
                <div className="flex items-center gap-4">
                    {member.avatar_url ? (
                        <Image
                            src={member.avatar_url}
                            alt={member.name}
                            width={64}
                            height={64}
                            className="rounded-full"
                        />
                    ) : (
                        <div className="w-16 h-16 rounded-full bg-neutral-200 dark:bg-neutral-700 flex items-center justify-center text-neutral-500 dark:text-neutral-400 text-xl font-bold">
                            {member.name.charAt(0).toUpperCase()}
                        </div>
                    )}
                    <div>
                        {/* 詳情頁的標題就是會員名稱，不另外套 PageHeader（麵包屑已標示所在位置） */}
                        <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">{member.name}</h1>
                        <p className="text-neutral-500 dark:text-neutral-400 text-sm">ID：{member.id}</p>
                    </div>
                </div>

                <div className="overflow-x-auto">
                    <AdminTable>
                        <tbody>
                            <tr>
                                <AdminTh className="bg-neutral-100 dark:bg-neutral-800 w-1/3">Email</AdminTh>
                                <AdminTd className="break-all">{member.email ?? '—'}</AdminTd>
                            </tr>
                            <tr>
                                <AdminTh className="bg-neutral-100 dark:bg-neutral-800">註冊時間</AdminTh>
                                <AdminTd>{formatDateTime(member.created_at)}</AdminTd>
                            </tr>
                            <tr>
                                <AdminTh className="bg-neutral-100 dark:bg-neutral-800">OAuth 綁定</AdminTh>
                                <AdminTd>
                                    <div className="flex gap-2 flex-wrap">
                                        {member.providers.length > 0 ? member.providers.map(p => (
                                            <span key={p} className="px-2 py-1 bg-primary-100 dark:bg-primary-900 text-primary-800 dark:text-primary-200 rounded text-sm font-medium">
                                                {p}
                                            </span>
                                        )) : <span className="text-neutral-500 dark:text-neutral-400">—</span>}
                                    </div>
                                </AdminTd>
                            </tr>
                        </tbody>
                    </AdminTable>
                </div>
            </div>
        </div>
    );
}

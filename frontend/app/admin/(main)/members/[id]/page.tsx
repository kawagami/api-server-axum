import { getMember } from "@/api/members";
import Image from "next/image";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Member Detail",
    description: "Member detail",
};

export default async function MemberDetailPage({ params }: { params: Promise<{ id: string }> }) {
    const id = Number((await params).id);
    const member = await getMember(id);

    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-3 sm:p-6">
            <div className="max-w-2xl mx-auto bg-white dark:bg-neutral-900 shadow-lg rounded-lg p-4 sm:p-6 space-y-4">
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
                        <h1 className="text-xl font-bold text-neutral-900 dark:text-neutral-100">{member.name}</h1>
                        <p className="text-neutral-500 dark:text-neutral-400 text-sm">ID: {member.id}</p>
                    </div>
                </div>

                <table className="w-full border-collapse border border-neutral-200 dark:border-neutral-700">
                    <tbody>
                        <tr>
                            <th className="border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800 w-1/3">Email</th>
                            <td className="border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-neutral-900 dark:text-neutral-100 break-all">{member.email ?? '—'}</td>
                        </tr>
                        <tr>
                            <th className="border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800">Created At</th>
                            <td className="border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-neutral-900 dark:text-neutral-100">
                                {new Date(member.created_at).toLocaleString()}
                            </td>
                        </tr>
                        <tr>
                            <th className="border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800">OAuth Providers</th>
                            <td className="border border-neutral-300 dark:border-neutral-700 px-4 py-2">
                                <div className="flex gap-2 flex-wrap">
                                    {member.providers.length > 0 ? member.providers.map(p => (
                                        <span key={p} className="px-2 py-1 bg-primary-100 dark:bg-primary-900 text-primary-800 dark:text-primary-200 rounded text-sm font-medium">
                                            {p}
                                        </span>
                                    )) : <span className="text-neutral-500 dark:text-neutral-400">—</span>}
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    );
}

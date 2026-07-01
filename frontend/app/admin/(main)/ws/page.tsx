import { getWsConnections } from "@/api/ws";
import SaySomethingForm from "./say-something-form";
import type { Metadata } from "next";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";

export const metadata: Metadata = {
    title: "WS Management",
    description: "WebSocket connections management",
};

export default async function WsAdminPage() {
    const connections = await getWsConnections();

    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-6">
            <div className="max-w-4xl mx-auto flex flex-col gap-8">
                <section>
                    <h2 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100 mb-4">
                        Online Connections ({connections.length})
                    </h2>
                    <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                        <AdminTable>
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh>Addr</AdminTh>
                                    <AdminTh>User Email</AdminTh>
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {connections.length === 0 ? (
                                    <tr>
                                        <td colSpan={2} className="border border-neutral-300 dark:border-neutral-700 px-4 py-4 text-center text-neutral-500 dark:text-neutral-400">
                                            No active connections
                                        </td>
                                    </tr>
                                ) : (
                                    connections.map((conn) => (
                                        <AdminRow key={conn.addr}>
                                            <AdminTd className="font-mono text-sm">
                                                {conn.addr}
                                            </AdminTd>
                                            <AdminTd className="text-sm">
                                                {conn.user_email ?? <span className="text-neutral-400">anonymous</span>}
                                            </AdminTd>
                                        </AdminRow>
                                    ))
                                )}
                            </tbody>
                        </AdminTable>
                    </div>
                </section>

                <section>
                    <h2 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100 mb-4">
                        Say Something to Someone
                    </h2>
                    <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg p-6">
                        <SaySomethingForm />
                    </div>
                </section>
            </div>
        </div>
    );
}

import type { Metadata } from "next";
import PageHeader from "@/components/admin/page-header";
import { requirePermission } from "@/libs/admin-permissions";
import TransferPanel from "./transfer-panel";

export const metadata: Metadata = {
    title: "檔案直傳",
    description: "手機與電腦之間直接傳送圖片、影片",
};

// 權限沿用 ws:read：WS 的 admin 身分（ticket）本來就要求它，後端只在同帳號的 admin 連線之間轉發 signaling
export default async function TransferPage() {
    await requirePermission("ws:read");

    return (
        <div className="w-full flex flex-col gap-4">
            <PageHeader title="檔案直傳" description="同一個帳號的兩台裝置都打開這一頁，即可互傳檔案" />
            <TransferPanel />
        </div>
    );
}

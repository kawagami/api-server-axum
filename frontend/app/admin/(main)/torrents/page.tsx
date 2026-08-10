import { Suspense } from "react";
import Link from "next/link";
import type { Metadata } from "next";
import { getTorrents, getTorrentStorage } from "@/api/torrents";
import { ListTableSkeleton } from "@/components/loading/table-skeleton";
import PageHeader from "@/components/admin/page-header";
import { TORRENT_STATUS_LABEL } from "@/libs/badge-styles";
import TorrentManager from "./torrent-manager";
import { requirePermission } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "Torrent 下載",
    description: "magnet 任務與檔案下載",
};

const PER_PAGE = 50;
const STATUS_TABS = ["", "pending", "downloading", "completed", "failed"];
const SKELETON_HEADERS = ["名稱", "狀態", "大小", "進度", "建立時間", "操作"];

function buildHref(status: string, page = 1) {
    const params = new URLSearchParams();
    if (status) params.append("status", status);
    if (page > 1) params.append("page", String(page));
    const qs = params.toString();
    return `/admin/torrents${qs ? `?${qs}` : ""}`;
}

async function TorrentContent({ status, page }: { status: string; page: number }) {
    const [{ data, total }, storage] = await Promise.all([
        getTorrents(status || null, page, PER_PAGE),
        getTorrentStorage().catch(() => null),
    ]);
    return (
        <TorrentManager
            initialTorrents={data}
            initialTotal={total}
            initialStorage={storage}
            status={status}
            page={page}
            perPage={PER_PAGE}
        />
    );
}

export default async function TorrentsPage({ searchParams }: { searchParams: Promise<{ status?: string; page?: string }> }) {
    await requirePermission("torrent:read");
    const { status: statusParam, page: pageStr } = await searchParams;
    const status = statusParam ?? "";
    const page = Math.max(1, Number(pageStr ?? 1) || 1);

    return (
        <div className="w-full flex min-h-0 flex-1 flex-col gap-4">
            <PageHeader title="Torrent 下載" />
            <div className="flex gap-2 flex-wrap">
                {STATUS_TABS.map((s) => {
                    const isActive = status === s;
                    return (
                        <Link
                            key={s || "all"}
                            href={buildHref(s)}
                            className={`px-4 py-2 rounded-lg border border-neutral-300 dark:border-neutral-600 text-sm transition-colors ${isActive
                                ? "bg-primary-500 text-white"
                                : "bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                                }`}
                        >
                            {s ? TORRENT_STATUS_LABEL[s] ?? s : "全部"}
                        </Link>
                    );
                })}
            </div>
            <Suspense key={`${status}-${page}`} fallback={<ListTableSkeleton headers={SKELETON_HEADERS} />}>
                <TorrentContent status={status} page={page} />
            </Suspense>
        </div>
    );
}

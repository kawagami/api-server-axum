import { Suspense } from "react";
import Link from "next/link";
import { getTorrents, getTorrentStorage } from "@/api/torrents";
import { ListTableSkeleton } from "@/components/loading/table-skeleton";
import TorrentManager from "./torrent-manager";

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
    const { status: statusParam, page: pageStr } = await searchParams;
    const status = statusParam ?? "";
    const page = Math.max(1, Number(pageStr ?? 1) || 1);

    return (
        <div className="w-full max-w-5xl mx-auto">
            <h1 className="text-xl font-bold mb-4 text-neutral-800 dark:text-white">Torrent 管理</h1>
            <div className="flex gap-2 mb-4 flex-wrap">
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
                            {s || "All"}
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

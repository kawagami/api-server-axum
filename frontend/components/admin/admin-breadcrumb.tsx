"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight } from "lucide-react";
import { adminNavGroups } from "@/components/admin/nav";

interface Crumb {
    label: string;
    href?: string;
}

// 動態段（uuid 等）截短顯示
function shorten(segment: string): string {
    const decoded = decodeURIComponent(segment);
    return decoded.length > 12 ? `${decoded.slice(0, 8)}…` : decoded;
}

/**
 * 從 adminNavGroups 反查當前路徑的麵包屑：
 * Admin / 群組（純文字，無對應頁）/ 選單項 / 額外段（[id] 等，純文字）。
 * 最後一層一律純文字；nav 查不到的路徑 fallback 顯示原始段。
 */
function resolveCrumbs(pathname: string): Crumb[] {
    const crumbs: Crumb[] = [{ label: "Admin", href: "/admin" }];
    if (pathname === "/admin") return crumbs;

    let best: { group: string; item: { label: string; href: string } } | null = null;
    for (const group of adminNavGroups) {
        for (const item of group.items) {
            if (pathname === item.href || pathname.startsWith(`${item.href}/`)) {
                if (!best || item.href.length > best.item.href.length) {
                    best = { group: group.label, item };
                }
            }
        }
    }

    if (!best) {
        for (const segment of pathname.slice("/admin/".length).split("/").filter(Boolean)) {
            crumbs.push({ label: shorten(segment) });
        }
        return crumbs;
    }

    crumbs.push({ label: best.group });
    const isLeaf = pathname === best.item.href;
    crumbs.push({ label: best.item.label, href: isLeaf ? undefined : best.item.href });
    if (!isLeaf) {
        for (const segment of pathname.slice(best.item.href.length + 1).split("/").filter(Boolean)) {
            crumbs.push({ label: shorten(segment) });
        }
    }
    return crumbs;
}

export default function AdminBreadcrumb({ className = "" }: { className?: string }) {
    const pathname = usePathname();
    const crumbs = resolveCrumbs(pathname);

    return (
        <nav aria-label="麵包屑" className={`flex items-center gap-1 text-sm ${className}`}>
            {crumbs.map((crumb, i) => {
                const isLast = i === crumbs.length - 1;
                return (
                    <span key={i} className="flex items-center gap-1 min-w-0">
                        {i > 0 && <ChevronRight size={14} className="shrink-0 text-neutral-400 dark:text-neutral-600" />}
                        {crumb.href && !isLast ? (
                            <Link
                                href={crumb.href}
                                className="text-neutral-500 dark:text-neutral-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                            >
                                {crumb.label}
                            </Link>
                        ) : (
                            <span
                                className={`truncate ${isLast
                                    ? "text-neutral-800 dark:text-white font-medium"
                                    : "text-neutral-500 dark:text-neutral-400"
                                    }`}
                                aria-current={isLast ? "page" : undefined}
                            >
                                {crumb.label}
                            </span>
                        )}
                    </span>
                );
            })}
        </nav>
    );
}

"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight, Menu, X, LogOut } from "lucide-react";
import { clearSession } from "@/app/admin/login/actions";
import { stopTokenRefresh } from "@/libs/token-refresh";
import { adminNavGroups as groups } from "@/components/admin/nav";

function SidebarContent({ pathname, onNavigate }: { pathname: string; onNavigate?: () => void }) {
    const [openGroups, setOpenGroups] = useState<Record<string, boolean>>(() =>
        Object.fromEntries(groups.map(g => [
            g.label,
            g.items.some(item => pathname.startsWith(item.href)),
        ]))
    );

    // sidebar 不隨 client 導航重新 mount；pathname 變動時於 render 期同步展開當前路由所在分組（單開）
    const [prevPathname, setPrevPathname] = useState(pathname);
    if (pathname !== prevPathname) {
        setPrevPathname(pathname);
        setOpenGroups(
            Object.fromEntries(groups.map(g => [
                g.label,
                g.items.some(item => pathname.startsWith(item.href)),
            ]))
        );
    }

    const toggle = (label: string) =>
        setOpenGroups(prev =>
            Object.fromEntries(
                groups.map(g => [g.label, g.label === label ? !prev[label] : false])
            )
        );

    return (
        <div className="flex flex-col h-full">
            <div className="flex-1 overflow-y-auto py-4 space-y-1">
                {groups.map(group => {
                    const Icon = group.icon;
                    const isOpen = openGroups[group.label];
                    const hasActive = group.items.some(item => pathname.startsWith(item.href));

                    return (
                        <div key={group.label}>
                            <button
                                onClick={() => toggle(group.label)}
                                className={`w-full flex items-center justify-between px-4 py-2 text-sm font-semibold rounded-lg transition-colors
                                    ${hasActive
                                        ? "text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-900/30"
                                        : "text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                                    }`}
                            >
                                <span className="flex items-center gap-2">
                                    <Icon size={16} />
                                    {group.label}
                                </span>
                                <ChevronRight
                                    size={14}
                                    className={`transition-transform duration-200 ease-out motion-reduce:transition-none ${isOpen ? "rotate-90" : ""}`}
                                />
                            </button>
                            <div
                                className={`grid transition-[grid-template-rows] duration-200 ease-out motion-reduce:transition-none ${isOpen ? "grid-rows-[1fr]" : "grid-rows-[0fr]"}`}
                            >
                                <div className="overflow-hidden">
                                    <div className="mt-1 ml-4 space-y-1">
                                        {group.items.map(item => {
                                            const isActive = pathname.startsWith(item.href);
                                            return (
                                                <Link
                                                    key={item.href}
                                                    href={item.href}
                                                    onClick={onNavigate}
                                                    tabIndex={isOpen ? 0 : -1}
                                                    className={`block px-4 py-1.5 text-sm rounded-lg transition-colors
                                                        ${isActive
                                                            ? "text-primary-600 dark:text-primary-400 bg-primary-100 dark:bg-primary-900/50 font-medium"
                                                            : "text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                                                        }`}
                                                >
                                                    {item.label}
                                                </Link>
                                            );
                                        })}
                                    </div>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
            <div className="border-t border-neutral-200 dark:border-neutral-700 p-4">
                <button
                    onClick={() => {
                        stopTokenRefresh();
                        localStorage.removeItem('token');
                        clearSession();
                    }}
                    className="w-full flex items-center gap-2 px-4 py-2 text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                >
                    <LogOut size={16} />
                    登出
                </button>
            </div>
        </div>
    );
}

export default function AdminSidebar() {
    const pathname = usePathname();
    const [drawerOpen, setDrawerOpen] = useState(false);

    return (
        <>
            {/* Desktop sidebar */}
            <aside className="hidden sm:flex flex-col w-52 shrink-0 border-r border-neutral-200 dark:border-neutral-700 bg-white/60 dark:bg-neutral-900/60 h-screen sticky top-0">
                <SidebarContent pathname={pathname} />
            </aside>

            {/* Mobile: hamburger button */}
            <button
                className="sm:hidden fixed top-3 left-3 z-40 p-1.5 rounded-lg bg-white dark:bg-neutral-800 shadow border border-neutral-200 dark:border-neutral-700"
                onClick={() => setDrawerOpen(true)}
                aria-label="開啟選單"
            >
                <Menu size={20} />
            </button>

            {/* Mobile: overlay */}
            {drawerOpen && (
                <div
                    className="sm:hidden fixed inset-0 z-40 bg-black/40"
                    onClick={() => setDrawerOpen(false)}
                />
            )}

            {/* Mobile: drawer */}
            <aside
                className={`sm:hidden fixed top-0 left-0 z-50 h-full w-64 bg-white dark:bg-neutral-900 shadow-xl transition-transform duration-300
                    ${drawerOpen ? "translate-x-0" : "-translate-x-full"}`}
            >
                <div className="flex items-center justify-between px-4 py-3 border-b border-neutral-200 dark:border-neutral-700">
                    <span className="font-semibold text-neutral-800 dark:text-white">Admin</span>
                    <button onClick={() => setDrawerOpen(false)} aria-label="關閉選單">
                        <X size={20} />
                    </button>
                </div>
                <SidebarContent pathname={pathname} onNavigate={() => setDrawerOpen(false)} />
            </aside>
        </>
    );
}

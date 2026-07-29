"use client";

import { useCallback, useEffect, useId, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight, Menu, X, LogOut, ExternalLink, Search } from "lucide-react";
import { clearSession } from "@/app/admin/login/actions";
import { stopTokenRefresh } from "@/libs/token-refresh";
import useDialog from "@/hooks/useDialog";
import AdminCommandPalette from "@/components/admin/command-palette";
import ThemeButton from "@/components/theme-button";
import type { UserColorMode } from "@/libs/color-mode";
import { adminNavGroups, filterNavByPermissions, type AdminNavGroup } from "@/components/admin/nav";

export interface AdminIdentity {
    name: string;
    isSuperAdmin: boolean;
}

interface SidebarFooterProps {
    admin: AdminIdentity;
    colorMode: UserColorMode;
    defaultIsDark: boolean | null;
    onOpenPalette: () => void;
    onNavigate?: () => void;
}

/** 側欄底部：我是誰 + 深淺色切換 + 快速跳頁 + 回前台 + 登出 */
function SidebarFooter({ admin, colorMode, defaultIsDark, onOpenPalette, onNavigate }: SidebarFooterProps) {
    return (
        <div className="border-t border-neutral-200 dark:border-neutral-700 p-3 space-y-1">
            <div className="flex items-center gap-2 px-1 py-2">
                <span
                    aria-hidden
                    className="shrink-0 w-8 h-8 rounded-full bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300 grid place-content-center text-sm font-semibold"
                >
                    {admin.name.charAt(0).toUpperCase()}
                </span>
                <div className="min-w-0 flex-1">
                    <div className="truncate text-sm font-medium text-neutral-800 dark:text-neutral-100" title={admin.name}>
                        {admin.name}
                    </div>
                    <div className="text-xs text-neutral-500 dark:text-neutral-400">
                        {admin.isSuperAdmin ? "超級管理員" : "管理員"}
                    </div>
                </div>
                <ThemeButton initialMode={colorMode} defaultIsDark={defaultIsDark} />
            </div>

            <button
                onClick={() => {
                    onNavigate?.();
                    onOpenPalette();
                }}
                className="w-full flex items-center gap-2 px-4 py-2 text-sm text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded-lg transition-colors"
            >
                <Search size={16} />
                <span className="flex-1 text-left">快速跳頁</span>
                {/* 兩個平台的快捷鍵一起標，免得為了偵測 OS 而弄出 hydration 不一致 */}
                <kbd className="text-[10px] border border-neutral-200 dark:border-neutral-700 rounded px-1.5 py-0.5">⌘/Ctrl K</kbd>
            </button>

            <Link
                href="/"
                onClick={onNavigate}
                className="w-full flex items-center gap-2 px-4 py-2 text-sm text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 rounded-lg transition-colors"
            >
                <ExternalLink size={16} />
                回前台
            </Link>

            <button
                onClick={() => {
                    stopTokenRefresh();
                    clearSession();
                }}
                className="w-full flex items-center gap-2 px-4 py-2 text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
            >
                <LogOut size={16} />
                登出
            </button>
        </div>
    );
}

function SidebarContent({ groups, pathname, onNavigate }: { groups: AdminNavGroup[]; pathname: string; onNavigate?: () => void }) {
    const panelIdPrefix = useId();
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
        // footer 已拆成 aside 的兄弟節點，這裡改用 flex-1 min-h-0 讓選單自己捲、footer 不被推出畫面
        <div className="flex flex-col flex-1 min-h-0">
            <nav aria-label="後台選單" className="flex-1 overflow-y-auto py-4 space-y-1">
                {groups.map((group, groupIndex) => {
                    const Icon = group.icon;
                    const isOpen = openGroups[group.label];
                    const hasActive = group.items.some(item => pathname.startsWith(item.href));
                    const panelId = `${panelIdPrefix}-group-${groupIndex}`;

                    return (
                        <div key={group.label}>
                            <button
                                onClick={() => toggle(group.label)}
                                aria-expanded={isOpen}
                                aria-controls={panelId}
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
                                id={panelId}
                                className={`grid transition-[grid-template-rows] duration-200 ease-out motion-reduce:transition-none ${isOpen ? "grid-rows-[1fr]" : "grid-rows-[0fr]"}`}
                            >
                                <div className="overflow-hidden">
                                    <ul className="mt-1 ml-4 space-y-1">
                                        {group.items.map(item => {
                                            const isActive = pathname.startsWith(item.href);
                                            return (
                                                <li key={item.href}>
                                                    <Link
                                                        href={item.href}
                                                        onClick={onNavigate}
                                                        tabIndex={isOpen ? 0 : -1}
                                                        aria-current={isActive ? "page" : undefined}
                                                        className={`block px-4 py-1.5 text-sm rounded-lg transition-colors
                                                            ${isActive
                                                                ? "text-primary-600 dark:text-primary-400 bg-primary-100 dark:bg-primary-900/50 font-medium"
                                                                : "text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                                                            }`}
                                                    >
                                                        {item.label}
                                                    </Link>
                                                </li>
                                            );
                                        })}
                                    </ul>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </nav>
        </div>
    );
}

export default function AdminSidebar({
    admin,
    permissions,
    enabledFeatures,
    colorMode,
    defaultIsDark,
}: {
    admin: AdminIdentity;
    permissions: string[];
    enabledFeatures: string[] | null;
    colorMode: UserColorMode;
    defaultIsDark: boolean | null;
}) {
    const pathname = usePathname();
    const [drawerOpen, setDrawerOpen] = useState(false);
    const [paletteOpen, setPaletteOpen] = useState(false);
    const groups = filterNavByPermissions(adminNavGroups, permissions, enabledFeatures);

    const closeDrawer = useCallback(() => setDrawerOpen(false), []);
    const closePalette = useCallback(() => setPaletteOpen(false), []);
    // Esc 關閉、背景不捲動、焦點鎖在抽屜內、關閉後焦點回到漢堡鈕
    const drawerRef = useDialog<HTMLElement>(drawerOpen, closeDrawer);

    // ⌘K / Ctrl+K 開快速跳頁。preventDefault 是為了蓋掉瀏覽器自己的搜尋列快捷
    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
                e.preventDefault();
                setPaletteOpen(true);
            }
        };
        document.addEventListener("keydown", onKeyDown);
        return () => document.removeEventListener("keydown", onKeyDown);
    }, []);

    return (
        <>
            {/* Desktop sidebar */}
            <aside className="hidden sm:flex flex-col w-52 shrink-0 border-r border-neutral-200 dark:border-neutral-700 bg-white/60 dark:bg-neutral-900/60 h-screen sticky top-0">
                <div className="px-4 py-3 border-b border-neutral-200 dark:border-neutral-700">
                    <Link
                        href="/admin"
                        className="font-semibold text-neutral-800 dark:text-white hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                    >
                        Admin
                    </Link>
                </div>
                <SidebarContent groups={groups} pathname={pathname} />
                <SidebarFooter
                    admin={admin}
                    colorMode={colorMode}
                    defaultIsDark={defaultIsDark}
                    onOpenPalette={() => setPaletteOpen(true)}
                />
            </aside>

            {/* Mobile: hamburger button */}
            <button
                className="sm:hidden fixed top-3 left-3 z-40 p-1.5 rounded-lg bg-white dark:bg-neutral-800 shadow border border-neutral-200 dark:border-neutral-700"
                onClick={() => setDrawerOpen(true)}
                aria-label="開啟選單"
                aria-expanded={drawerOpen}
            >
                <Menu size={20} />
            </button>

            {/* Mobile: overlay */}
            {drawerOpen && (
                <div
                    className="sm:hidden fixed inset-0 z-40 bg-black/40"
                    onClick={closeDrawer}
                />
            )}

            {/* Mobile: drawer —— 常駐 DOM 靠 transform 滑出，關閉時用 inert 讓它退出 tab 順序與無障礙樹 */}
            <aside
                ref={drawerRef}
                role="dialog"
                aria-modal="true"
                aria-label="後台選單"
                inert={!drawerOpen}
                className={`sm:hidden flex flex-col fixed top-0 left-0 z-50 h-[100dvh] w-64 bg-white dark:bg-neutral-900 shadow-xl transition-transform duration-300 motion-reduce:transition-none
                    ${drawerOpen ? "translate-x-0" : "-translate-x-full"}`}
            >
                <div className="flex items-center justify-between px-4 py-3 border-b border-neutral-200 dark:border-neutral-700">
                    <Link
                        href="/admin"
                        onClick={closeDrawer}
                        className="font-semibold text-neutral-800 dark:text-white hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                    >
                        Admin
                    </Link>
                    <button onClick={closeDrawer} aria-label="關閉選單">
                        <X size={20} />
                    </button>
                </div>
                <SidebarContent groups={groups} pathname={pathname} onNavigate={closeDrawer} />
                <SidebarFooter
                    admin={admin}
                    colorMode={colorMode}
                    defaultIsDark={defaultIsDark}
                    onOpenPalette={() => setPaletteOpen(true)}
                    onNavigate={closeDrawer}
                />
            </aside>

            {/* 快速跳頁：與側邊欄共用同一份（已依權限過濾的）選單來源 */}
            {paletteOpen && <AdminCommandPalette groups={groups} onClose={closePalette} />}
        </>
    );
}

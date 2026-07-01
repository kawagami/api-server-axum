"use client";

import { useState, useEffect, useRef } from 'react';
import { Link, usePathname } from '@/i18n/navigation';
import ThemeButton from "@/components/theme-button";
import KawaLogo from "@/components/kawa-logo";
import { logout } from '@/actions/auth';
import { LayoutDashboard, User, Bell, ChevronDown, X, Menu, TrendingUp, Wallet, ReceiptText, Ticket, type LucideIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import LocaleSwitcher from '@/components/locale-switcher';

import type { UserColorMode } from "@/libs/color-mode";

interface HeaderProps {
    member: { id: string } | null
    colorMode: UserColorMode
    defaultIsDark: boolean | null
}

// 工具選單與會員選單的單一來源；新增工具只要加一行
const TOOLS = [
    { href: "/tools/new-password", labelKey: "toolNewPassword" },
    { href: "/tools/convert-text", labelKey: "toolConvertText" },
    { href: "/tools/countdown", labelKey: "toolCountdown" },
    { href: "/tools/roster", labelKey: "toolRoster" },
    { href: "/tools/alarm", labelKey: "toolAlarm" },
    { href: "/tools/hourly-chime", labelKey: "toolHourlyChime" },
] as const;

// 遊戲選單；新增遊戲只要加一行
const GAMES = [
    { href: "/games/chess", labelKey: "gameChess" },
    { href: "/games/western-chess", labelKey: "gameWesternChess" },
    { href: "/games/gomoku", labelKey: "gameGomoku" },
    { href: "/games/go", labelKey: "gameGo" },
    { href: "/games/banqi", labelKey: "gameBanqi" },
    { href: "/games/avalon", labelKey: "gameAvalon" },
    { href: "/games/farm", labelKey: "gameFarm" },
    { href: "/games/metal-slug", labelKey: "gameMetalSlug" },
] as const;

const MEMBER_LINKS: ReadonlyArray<{ href: string; labelKey: string; icon: LucideIcon }> = [
    { href: "/dashboard", labelKey: "dashboard", icon: LayoutDashboard },
    { href: "/profile", labelKey: "profile", icon: User },
    { href: "/dashboard/notifications", labelKey: "notifications", icon: Bell },
    { href: "/portfolio", labelKey: "portfolio", icon: TrendingUp },
    { href: "/ledger", labelKey: "ledger", icon: Wallet },
    { href: "/invoices", labelKey: "invoices", icon: ReceiptText },
    { href: "/lotto", labelKey: "lotto", icon: Ticket },
];

const navLinkClass = "block px-4 rounded hover:text-primary-600 dark:hover:text-primary-300 hover:underline underline-offset-4 focus:outline-none focus:ring-2 focus:ring-primary-400 whitespace-nowrap";
const navTriggerClass = navLinkClass.replace('block', 'inline-flex items-center gap-1');
const activeNavClass = "text-primary-600 dark:text-primary-300 font-medium";
const dropdownItemClass = "flex items-center gap-2 px-4 py-2 hover:bg-neutral-100 dark:hover:bg-neutral-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary-400";
const mobileItemClass = "px-4 py-2 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-800 focus:outline-none focus:ring-2 focus:ring-primary-400";

function DesktopDropdown({ isOpen, align = 'left', children }: { isOpen: boolean; align?: 'left' | 'right'; children: React.ReactNode }) {
    return (
        <div
            className={`absolute ${align === 'left' ? 'left-0' : 'right-0'} mt-1 bg-white dark:bg-neutral-800 shadow-lg rounded-md overflow-hidden z-10 min-w-[120px]
                origin-top transition-all duration-150 ease-out motion-reduce:transition-none
                ${isOpen ? "opacity-100 translate-y-0 visible" : "opacity-0 -translate-y-1 invisible"}`}
        >
            {children}
        </div>
    );
}

export default function Header({ member, colorMode, defaultIsDark }: HeaderProps) {
    const [isOpen, setIsOpen] = useState(false);
    const [isResourcesOpen, setIsResourcesOpen] = useState(false);
    const [isGamesOpen, setIsGamesOpen] = useState(false);
    const [isMemberOpen, setIsMemberOpen] = useState(false);
    const t = useTranslations('Header');
    const pathname = usePathname();
    const navRef = useRef<HTMLElement>(null);

    const isBlogActive = pathname.startsWith('/blogs');
    const isNotesActive = pathname.startsWith('/hackmd-notes');
    const isAboutActive = pathname.startsWith('/about');
    const isToolsActive = TOOLS.some(({ href }) => pathname.startsWith(href));
    const isGamesActive = GAMES.some(({ href }) => pathname.startsWith(href));

    const closeAll = () => {
        setIsOpen(false);
        setIsResourcesOpen(false);
        setIsGamesOpen(false);
        setIsMemberOpen(false);
    };

    const closeDesktopDropdowns = () => {
        setIsResourcesOpen(false);
        setIsGamesOpen(false);
        setIsMemberOpen(false);
    };

    // Escape 關閉所有選單；點桌面 nav 外部關閉下拉（行動選單有自己的 backdrop）
    useEffect(() => {
        if (!isOpen && !isResourcesOpen && !isGamesOpen && !isMemberOpen) return;
        const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') closeAll(); };
        const onPointerDown = (e: PointerEvent) => {
            if (isOpen) return; // 行動選單開啟時交給 backdrop 處理
            if (navRef.current && !navRef.current.contains(e.target as Node)) {
                closeDesktopDropdowns();
            }
        };
        window.addEventListener('keydown', onKey);
        document.addEventListener('pointerdown', onPointerDown);
        return () => {
            window.removeEventListener('keydown', onKey);
            document.removeEventListener('pointerdown', onPointerDown);
        };
    }, [isOpen, isResourcesOpen, isGamesOpen, isMemberOpen]);

    return (
        <>
            <header className="min-h-[50px] flex items-center justify-between px-4 relative z-50">
                <div className="flex items-center flex-shrink-0">
                    <Link href="/" className="block px-2" aria-label={t('backToHome')} onClick={closeAll}>
                        <KawaLogo width={100} height={40} />
                    </Link>
                </div>

                {/* Desktop nav */}
                <nav ref={navRef} className="hidden md:flex items-center gap-2">
                    <Link href="/blogs" aria-label={t('blog')} className={`${navLinkClass} ${isBlogActive ? activeNavClass : ''}`}>{t('blog')}</Link>
                    <Link href="/hackmd-notes" aria-label={t('notes')} className={`${navLinkClass} ${isNotesActive ? activeNavClass : ''}`}>{t('notes')}</Link>
                    <div className="relative">
                        <button
                            className={`${navTriggerClass} ${isToolsActive ? activeNavClass : ''}`}
                            aria-label={t('openToolsMenu')}
                            aria-expanded={isResourcesOpen}
                            onClick={() => setIsResourcesOpen(o => !o)}
                        >
                            {t('tools')}
                            <ChevronDown size={14} className={`transition-transform duration-200 motion-reduce:transition-none ${isResourcesOpen ? 'rotate-180' : ''}`} />
                        </button>
                        <DesktopDropdown isOpen={isResourcesOpen}>
                            {TOOLS.map(({ href, labelKey }) => (
                                <Link key={href} href={href} tabIndex={isResourcesOpen ? 0 : -1} className={dropdownItemClass} onClick={() => setIsResourcesOpen(false)}>
                                    {t(labelKey)}
                                </Link>
                            ))}
                        </DesktopDropdown>
                    </div>
                    <div className="relative">
                        <button
                            className={`${navTriggerClass} ${isGamesActive ? activeNavClass : ''}`}
                            aria-label={t('openGamesMenu')}
                            aria-expanded={isGamesOpen}
                            onClick={() => setIsGamesOpen(o => !o)}
                        >
                            {t('games')}
                            <ChevronDown size={14} className={`transition-transform duration-200 motion-reduce:transition-none ${isGamesOpen ? 'rotate-180' : ''}`} />
                        </button>
                        <DesktopDropdown isOpen={isGamesOpen}>
                            {GAMES.map(({ href, labelKey }) => (
                                <Link key={href} href={href} tabIndex={isGamesOpen ? 0 : -1} className={dropdownItemClass} onClick={() => setIsGamesOpen(false)}>
                                    {t(labelKey)}
                                </Link>
                            ))}
                        </DesktopDropdown>
                    </div>
                    <Link href="/about" aria-label={t('about')} className={`${navLinkClass} ${isAboutActive ? activeNavClass : ''}`}>{t('about')}</Link>
                    <LocaleSwitcher />
                    <ThemeButton initialMode={colorMode} defaultIsDark={defaultIsDark} />
                    {member ? (
                        <div className="relative">
                            <button
                                className="flex items-center gap-1 px-4 rounded hover:text-primary-600 dark:hover:text-primary-300 focus:outline-none focus:ring-2 focus:ring-primary-400"
                                aria-label={t('openMemberMenu')}
                                aria-expanded={isMemberOpen}
                                onClick={() => setIsMemberOpen(o => !o)}
                            >
                                <User size={16} />
                                <ChevronDown size={14} className={`transition-transform duration-200 motion-reduce:transition-none ${isMemberOpen ? 'rotate-180' : ''}`} />
                            </button>
                            <DesktopDropdown isOpen={isMemberOpen} align="right">
                                {MEMBER_LINKS.map(({ href, labelKey, icon: Icon }) => (
                                    <Link key={href} href={href} tabIndex={isMemberOpen ? 0 : -1} className={`${dropdownItemClass} text-sm`} onClick={() => setIsMemberOpen(false)}>
                                        <Icon size={14} />
                                        {t(labelKey)}
                                    </Link>
                                ))}
                                <form action={logout}>
                                    <button type="submit" tabIndex={isMemberOpen ? 0 : -1} className={`w-full ${dropdownItemClass} text-sm text-red-500 dark:text-red-400`}>
                                        {t('logout')}
                                    </button>
                                </form>
                            </DesktopDropdown>
                        </div>
                    ) : (
                        <Link href="/login" className={navLinkClass}>{t('login')}</Link>
                    )}
                </nav>

                {/* Mobile hamburger */}
                <button
                    className="md:hidden p-2 rounded focus:outline-none focus:ring-2 focus:ring-primary-400"
                    onClick={() => setIsOpen(o => !o)}
                    aria-label={isOpen ? t('closeMenu') : t('openMenu')}
                >
                    {isOpen ? <X size={24} /> : <Menu size={24} />}
                </button>
            </header>

            {/* Mobile nav overlay */}
            {isOpen && (
                <>
                    <div className="md:hidden fixed inset-0 z-30 bg-black/40" onClick={closeAll} aria-hidden="true" />
                    <nav className="md:hidden fixed top-[50px] left-0 right-0 z-40 bg-white dark:bg-neutral-900 shadow-lg border-t border-neutral-200 dark:border-neutral-700 flex flex-col p-4 gap-1">
                        <Link href="/blogs" className={`${mobileItemClass} ${isBlogActive ? activeNavClass : ''}`} onClick={closeAll}>{t('blog')}</Link>
                        <Link href="/hackmd-notes" className={`${mobileItemClass} ${isNotesActive ? activeNavClass : ''}`} onClick={closeAll}>{t('notes')}</Link>

                        <button
                            className={`${mobileItemClass} flex items-center justify-between w-full text-left ${isToolsActive ? activeNavClass : ''}`}
                            aria-expanded={isResourcesOpen}
                            onClick={() => setIsResourcesOpen(o => !o)}
                        >
                            {t('tools')}
                            <ChevronDown size={14} className={`transition-transform ${isResourcesOpen ? 'rotate-180' : ''}`} />
                        </button>
                        {isResourcesOpen && (
                            <div className="ml-4 flex flex-col gap-1">
                                {TOOLS.map(({ href, labelKey }) => (
                                    <Link key={href} href={href} className={`${mobileItemClass} text-sm`} onClick={closeAll}>
                                        {t(labelKey)}
                                    </Link>
                                ))}
                            </div>
                        )}

                        <button
                            className={`${mobileItemClass} flex items-center justify-between w-full text-left ${isGamesActive ? activeNavClass : ''}`}
                            aria-expanded={isGamesOpen}
                            onClick={() => setIsGamesOpen(o => !o)}
                        >
                            {t('games')}
                            <ChevronDown size={14} className={`transition-transform ${isGamesOpen ? 'rotate-180' : ''}`} />
                        </button>
                        {isGamesOpen && (
                            <div className="ml-4 flex flex-col gap-1">
                                {GAMES.map(({ href, labelKey }) => (
                                    <Link key={href} href={href} className={`${mobileItemClass} text-sm`} onClick={closeAll}>
                                        {t(labelKey)}
                                    </Link>
                                ))}
                            </div>
                        )}

                        <Link href="/about" className={`${mobileItemClass} ${isAboutActive ? activeNavClass : ''}`} onClick={closeAll}>{t('about')}</Link>
                        <div className="px-4 py-2">
                            <ThemeButton initialMode={colorMode} defaultIsDark={defaultIsDark} />
                        </div>
                        <div className="px-4 py-2">
                            <LocaleSwitcher />
                        </div>

                        {member ? (
                            <>
                                {MEMBER_LINKS.map(({ href, labelKey, icon: Icon }) => (
                                    <Link key={href} href={href} className={`${mobileItemClass} flex items-center gap-2 text-sm`} onClick={closeAll}>
                                        <Icon size={14} />
                                        {t(labelKey)}
                                    </Link>
                                ))}
                                <form action={logout}>
                                    <button type="submit" className={`w-full ${mobileItemClass} flex items-center gap-2 text-sm text-red-500 dark:text-red-400`}>
                                        {t('logout')}
                                    </button>
                                </form>
                            </>
                        ) : (
                            <Link href="/login" className={mobileItemClass} onClick={closeAll}>{t('login')}</Link>
                        )}
                    </nav>
                </>
            )}
        </>
    );
}

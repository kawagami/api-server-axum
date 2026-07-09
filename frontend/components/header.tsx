"use client";

import { useState, useEffect, useRef } from 'react';
import { Link, usePathname } from '@/i18n/navigation';
import ThemeButton from "@/components/theme-button";
import KawaLogo from "@/components/kawa-logo";
import { logout } from '@/actions/auth';
import { User, ChevronDown, X, Menu } from 'lucide-react';
import { useTranslations } from 'next-intl';
import LocaleSwitcher from '@/components/locale-switcher';
import { TOOLS, GAMES, MEMBER_LINKS } from '@/libs/site-nav';

import type { UserColorMode } from "@/libs/color-mode";

interface HeaderProps {
    member: { id: string } | null
    colorMode: UserColorMode
    defaultIsDark: boolean | null
}

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
    const isVocabActive = pathname.startsWith('/vocab');
    const isNotesActive = pathname.startsWith('/hackmd-notes');
    const isAboutActive = pathname.startsWith('/about');
    const isToolsActive = pathname.startsWith('/tools');
    const isGamesActive = pathname.startsWith('/games');

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
                    <Link href="/vocab" aria-label={t('vocab')} className={`${navLinkClass} ${isVocabActive ? activeNavClass : ''}`}>{t('vocab')}</Link>
                    <Link href="/hackmd-notes" aria-label={t('notes')} className={`${navLinkClass} ${isNotesActive ? activeNavClass : ''}`}>{t('notes')}</Link>
                    <div
                        className="relative"
                        onMouseEnter={() => setIsResourcesOpen(true)}
                        onMouseLeave={() => setIsResourcesOpen(false)}
                    >
                        <span className={`${navTriggerClass} ${isToolsActive ? activeNavClass : ''}`}>
                            <Link href="/tools" aria-label={t('tools')} className="focus:outline-none focus:ring-2 focus:ring-primary-400 rounded" onClick={closeDesktopDropdowns}>
                                {t('tools')}
                            </Link>
                            <button
                                aria-label={t('openToolsMenu')}
                                aria-expanded={isResourcesOpen}
                                onClick={() => setIsResourcesOpen(o => !o)}
                                className="focus:outline-none focus:ring-2 focus:ring-primary-400 rounded"
                            >
                                <ChevronDown size={14} className={`transition-transform duration-200 motion-reduce:transition-none ${isResourcesOpen ? 'rotate-180' : ''}`} />
                            </button>
                        </span>
                        <DesktopDropdown isOpen={isResourcesOpen}>
                            {TOOLS.map(({ href, labelKey }) => (
                                <Link key={href} href={href} tabIndex={isResourcesOpen ? 0 : -1} className={dropdownItemClass} onClick={() => setIsResourcesOpen(false)}>
                                    {t(labelKey)}
                                </Link>
                            ))}
                        </DesktopDropdown>
                    </div>
                    <div
                        className="relative"
                        onMouseEnter={() => setIsGamesOpen(true)}
                        onMouseLeave={() => setIsGamesOpen(false)}
                    >
                        <span className={`${navTriggerClass} ${isGamesActive ? activeNavClass : ''}`}>
                            <Link href="/games" aria-label={t('games')} className="focus:outline-none focus:ring-2 focus:ring-primary-400 rounded" onClick={closeDesktopDropdowns}>
                                {t('games')}
                            </Link>
                            <button
                                aria-label={t('openGamesMenu')}
                                aria-expanded={isGamesOpen}
                                onClick={() => setIsGamesOpen(o => !o)}
                                className="focus:outline-none focus:ring-2 focus:ring-primary-400 rounded"
                            >
                                <ChevronDown size={14} className={`transition-transform duration-200 motion-reduce:transition-none ${isGamesOpen ? 'rotate-180' : ''}`} />
                            </button>
                        </span>
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
                        <div
                            className="relative"
                            onMouseEnter={() => setIsMemberOpen(true)}
                            onMouseLeave={() => setIsMemberOpen(false)}
                        >
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
                        <Link href="/vocab" className={`${mobileItemClass} ${isVocabActive ? activeNavClass : ''}`} onClick={closeAll}>{t('vocab')}</Link>
                        <Link href="/hackmd-notes" className={`${mobileItemClass} ${isNotesActive ? activeNavClass : ''}`} onClick={closeAll}>{t('notes')}</Link>

                        <div className="flex items-center">
                            <Link href="/tools" className={`${mobileItemClass} flex-1 ${isToolsActive ? activeNavClass : ''}`} onClick={closeAll}>
                                {t('tools')}
                            </Link>
                            <button
                                className={mobileItemClass}
                                aria-label={t('openToolsMenu')}
                                aria-expanded={isResourcesOpen}
                                onClick={() => setIsResourcesOpen(o => !o)}
                            >
                                <ChevronDown size={14} className={`transition-transform ${isResourcesOpen ? 'rotate-180' : ''}`} />
                            </button>
                        </div>
                        {isResourcesOpen && (
                            <div className="ml-4 flex flex-col gap-1">
                                {TOOLS.map(({ href, labelKey }) => (
                                    <Link key={href} href={href} className={`${mobileItemClass} text-sm`} onClick={closeAll}>
                                        {t(labelKey)}
                                    </Link>
                                ))}
                            </div>
                        )}

                        <div className="flex items-center">
                            <Link href="/games" className={`${mobileItemClass} flex-1 ${isGamesActive ? activeNavClass : ''}`} onClick={closeAll}>
                                {t('games')}
                            </Link>
                            <button
                                className={mobileItemClass}
                                aria-label={t('openGamesMenu')}
                                aria-expanded={isGamesOpen}
                                onClick={() => setIsGamesOpen(o => !o)}
                            >
                                <ChevronDown size={14} className={`transition-transform ${isGamesOpen ? 'rotate-180' : ''}`} />
                            </button>
                        </div>
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

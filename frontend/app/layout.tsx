import "./globals.css";
import { WsProvider } from "@/libs/ws-context";
import ThemeBackground from "@/components/ThemeBackground";
import { getPublicSettings } from "@/api/settings";
import { resolveActiveTheme, normalizeRotation } from "@/libs/site-theme";
import { resolveDefaultColorMode } from "@/libs/color-mode";
import type { Metadata } from "next";
import { cookies } from "next/headers";

export const metadata: Metadata = {
    metadataBase: new URL("https://kawa.homes"),
    title: "Kawa's Blog",
    description: "kawa blog ongoing",
};

export default async function RootLayout({ children }: { children: React.ReactNode }) {
    const [cookieStore, publicSettings] = await Promise.all([cookies(), getPublicSettings()]);
    // 全站風格：site_theme 是具體主題 → 固定；是 'auto' → 依 theme_rotation + 當天星期（Asia/Taipei）輪播
    // layout 因 cookies() 為動態渲染、每 request 重算，跨午夜自動換主題不受 60s settings cache 影響
    const siteTheme = resolveActiveTheme(publicSettings.site_theme, normalizeRotation(publicSettings.theme_rotation));
    const jwt = cookieStore.get('session')?.value ?? null;

    // 深淺色：使用者 cookie ＞ admin 預設 ＞ 系統偏好
    const themeCookie = cookieStore.get('theme')?.value;   // dark / light / undefined
    const defaultMode = resolveDefaultColorMode(publicSettings.default_color_mode);
    const isDark = themeCookie ? themeCookie === 'dark' : defaultMode === 'dark';
    const followSystem = !themeCookie && defaultMode === 'system';

    return (
        <html lang="zh-TW" className={isDark ? 'dark' : ''} data-theme={siteTheme} suppressHydrationWarning>
            <head>
                {followSystem && (
                    <script dangerouslySetInnerHTML={{ __html: `(function(){try{if(window.matchMedia('(prefers-color-scheme: dark)').matches){document.documentElement.classList.add('dark');}}catch(e){}})();` }} />
                )}
            </head>
            <body className="bg-gradient-to-b from-primary-50 to-neutral-100 text-neutral-800 dark:from-primary-950 dark:to-neutral-900 dark:text-neutral-100">
                <WsProvider jwt={jwt} wsUrl={process.env.WS_URL ?? ''}>
                    <ThemeBackground theme={siteTheme} />
                    {children}
                </WsProvider>
            </body>
        </html>
    );
}

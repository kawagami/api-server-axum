import Footer from "@/components/footer";
import Header from "@/components/header";
import WsToast from "@/components/ws/ws-toast";
import PageTransition from "@/components/page-transition";
import { getPublicSettings } from "@/api/settings";
import { resolveDefaultColorMode, type UserColorMode } from "@/libs/color-mode";
import { resolveEnabledFeatures } from "@/libs/enabled-features";
import { cookies } from "next/headers";
import { getTranslations } from "next-intl/server";
import { jwtVerify } from "jose";

async function getMember(token: string | undefined): Promise<{ id: string } | null> {
    if (!token) return null
    try {
        const secret = new TextEncoder().encode(process.env.JWT_SECRET)
        const { payload } = await jwtVerify(token, secret)
        if (payload.role !== 'member') return null
        return { id: payload.sub as string }
    } catch {
        return null
    }
}

export default async function PublicLayout({ children }: { children: React.ReactNode }) {
    const cookieStore = await cookies();
    // getPublicSettings 與 root layout 同 URL 同參數，Next fetch cache 自動去重
    const [member, publicSettings] = await Promise.all([
        getMember(cookieStore.get('access_token')?.value),
        getPublicSettings(),
    ]);

    const themeCookie = cookieStore.get('theme')?.value;
    const colorMode: UserColorMode =
        themeCookie === 'dark' ? 'dark' : themeCookie === 'light' ? 'light' : 'auto';
    const defaultMode = resolveDefaultColorMode(publicSettings.default_color_mode);
    const defaultIsDark = defaultMode === 'system' ? null : defaultMode === 'dark';
    const enabledFeatures = resolveEnabledFeatures(publicSettings.enabled_features);

    const t = await getTranslations('Header');

    return (
        <>
            {/* 第一個可聚焦元素：鍵盤使用者不必逐一 Tab 過整組導航 */}
            <a href="#main-content" className="skip-link rounded-lg bg-primary-600 px-4 py-2 text-sm text-white shadow-lg">
                {t('skipToContent')}
            </a>
            <Header member={member} colorMode={colorMode} defaultIsDark={defaultIsDark} enabledFeatures={enabledFeatures} />
            {/* 不加 overflow-hidden：那會讓子層 sticky 失效，且頁面被迫各自開內捲區。
                上下 padding 一律由 PageShell 給，這裡只負責撐滿視窗高度。 */}
            <main id="main-content" className="min-h-[calc(100svh-50px-50px)] flex flex-col items-center justify-start">
                <PageTransition>{children}</PageTransition>
            </main>
            <Footer enabledFeatures={enabledFeatures} />
            <WsToast />
        </>
    );
}

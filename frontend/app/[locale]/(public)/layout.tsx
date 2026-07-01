import Footer from "@/components/footer";
import Header from "@/components/header";
import WsToast from "@/components/ws/ws-toast";
import { getPublicSettings } from "@/api/settings";
import { resolveDefaultColorMode, type UserColorMode } from "@/libs/color-mode";
import { cookies } from "next/headers";
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

    return (
        <>
            <Header member={member} colorMode={colorMode} defaultIsDark={defaultIsDark} />
            <main className="min-h-[calc(100svh-50px-50px)] overflow-hidden flex flex-col items-center justify-start pt-4">
                {children}
            </main>
            <Footer />
            <WsToast />
        </>
    );
}

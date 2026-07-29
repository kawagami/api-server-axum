import type { Metadata } from "next";

// page.tsx 是 client component（WebAuthn ceremony），metadata 只能掛在 layout
export const metadata: Metadata = {
    title: "Passkey 管理",
    description: "以指紋、臉部辨識或裝置密碼登入後台",
};

export default function PasskeysLayout({ children }: { children: React.ReactNode }) {
    return children;
}

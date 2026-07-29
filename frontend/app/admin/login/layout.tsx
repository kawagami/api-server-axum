import type { Metadata } from "next";

// page.tsx 是 client component（WebAuthn / useActionState），metadata 只能掛在 layout
export const metadata: Metadata = {
    title: "登入",
    description: "後台管理員登入",
};

export default function AdminLoginLayout({ children }: { children: React.ReactNode }) {
    return children;
}

import type { Metadata } from "next";

// page.tsx 是 client component（useActionState），metadata 只能掛在 layout
export const metadata: Metadata = {
    title: "修改密碼",
    description: "變更後台管理員密碼",
};

export default function ChangePasswordLayout({ children }: { children: React.ReactNode }) {
    return children;
}

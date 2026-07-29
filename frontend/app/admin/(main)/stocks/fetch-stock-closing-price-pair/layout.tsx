import type { Metadata } from "next";

// page.tsx 是 client component（useActionState），metadata 只能掛在 layout
export const metadata: Metadata = {
    title: "收盤價查詢",
    description: "查詢特定股票在指定區間的收盤價",
};

export default function ClosingPricePairLayout({ children }: { children: React.ReactNode }) {
    return children;
}

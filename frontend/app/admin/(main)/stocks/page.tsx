"use client";

import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";
import { Loader2 } from "lucide-react";

interface Link {
    href: string;
    label: string;
    color: 'green' | 'indigo' | 'blue';
}

const LINKS: Link[] = [
    { href: "/admin/stocks/list", label: "所有查詢後的資料", color: "green" },
    { href: "/admin/stocks/fetch-stock-closing-price-pair", label: "查詢特定區間特定股票歷史價格", color: "indigo" },
    { href: "/admin/stocks/get-unfinished-buyback-price-gap", label: "執行中的庫藏股", color: "green" },
    { href: "/admin/stocks/stock-day-all", label: "全市場行情", color: "blue" },
    { href: "/admin/stocks/get-buyback-plans", label: "庫藏股計畫", color: "blue" },
];

const COLOR_MAP: Record<string, string> = {
    green: "bg-primary-600 border-primary-700 hover:bg-primary-700",
    indigo: "bg-primary-600 border-primary-700 hover:bg-primary-700",
    blue: "bg-primary-600 border-primary-700 hover:bg-primary-700",
};

export default function Stocks() {
    const router = useRouter();
    const [isPending, startTransition] = useTransition();
    const [loadingHref, setLoadingHref] = useState<string | null>(null);

    function navigate(href: string) {
        setLoadingHref(href);
        startTransition(() => {
            router.push(href);
        });
    }

    return (
        <div className="w-4/5 max-h-[calc(100svh-180px)] overflow-auto p-6 bg-neutral-100 dark:bg-neutral-800">
            <div className="w-full flex flex-wrap justify-center gap-4">
                {LINKS.map(({ href, label, color }) => {
                    const isLoading = isPending && loadingHref === href;
                    const isDisabled = isPending;
                    return (
                        <button
                            key={href}
                            onClick={() => navigate(href)}
                            disabled={isDisabled}
                            className={`px-6 py-3 font-semibold text-white border-2 rounded-lg shadow-md transition duration-300 flex items-center gap-2 ${COLOR_MAP[color]} ${isDisabled ? "opacity-60 cursor-not-allowed" : "hover:scale-105 cursor-pointer"}`}
                        >
                            {isLoading && <Loader2 className="w-4 h-4 animate-spin" />}
                            {label}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}

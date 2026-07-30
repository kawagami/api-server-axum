"use client";

import { usePathname } from "@/i18n/navigation";

/**
 * 換頁淡入。原本只有 /tools 底下有（tools/layout.tsx），其他頁切過去是硬切，
 * 同一個站兩種換頁觀感；提到 (public) layout 之後全前台一致。
 * key={pathname} 讓每次換路徑重新掛載，動畫才會重播。
 */
export default function PageTransition({ children }: { children: React.ReactNode }) {
    const pathname = usePathname();

    return (
        <div key={pathname} className="w-full animate-fade-in">
            {children}
        </div>
    );
}

import type { Metadata } from "next";

export const metadata: Metadata = {
    // 各頁 metadata.title 只填短名（如「訪客留言」），這裡統一補後綴，
    // 分頁標題一眼分得出是後台的哪一頁，而不是全部顯示站台預設名稱
    title: {
        template: "%s｜後台",
        default: "後台",
    },
};

// admin-shell：後台專用的樣式作用域（見 globals.css 的統一焦點樣式）
export default function AdminLayout({ children }: { children: React.ReactNode }) {
    return (
        <div className="admin-shell min-h-screen flex flex-col">
            {children}
        </div>
    );
}

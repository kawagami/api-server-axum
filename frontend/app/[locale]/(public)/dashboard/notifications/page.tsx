import NotificationFeed from "@/components/ws/notification-feed";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

export async function generateMetadata(): Promise<Metadata> {
    // 標題沿用 header／dashboard 的導覽標籤，三處說法一致
    const t = await getTranslations("Header");
    return { title: t("notifications") };
}

export default function NotificationsPage() {
    return <NotificationFeed />;
}

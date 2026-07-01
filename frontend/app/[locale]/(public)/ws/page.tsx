import NotificationFeed from "@/components/ws/notification-feed";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Ws");
    return { title: t("title"), description: t("description") };
}

export default function WsPage() {
    return <NotificationFeed />;
}

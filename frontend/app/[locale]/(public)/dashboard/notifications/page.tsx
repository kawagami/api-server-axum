import NotificationFeed from "@/components/ws/notification-feed";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "通知",
};

export default function NotificationsPage() {
    return <NotificationFeed />;
}

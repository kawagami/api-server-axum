import MessagesClient from "./messages-client";
import type { Metadata } from "next";
import { requirePermission, getMyPermissions } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "訪客留言",
    description: "訪客從前台留言給站長的訊息",
};

export default async function MessagesPage() {
    await requirePermission("message:read");
    const canDelete = (await getMyPermissions()).includes("message:delete");
    return <MessagesClient canDelete={canDelete} />;
}

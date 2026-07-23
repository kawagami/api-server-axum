import BlogCommentsClient from "./blog-comments-client";
import type { Metadata } from "next";
import { requirePermission, getMyPermissions } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "部落格留言",
    description: "訪客與會員在文章下的留言",
};

export default async function BlogCommentsPage() {
    await requirePermission("comment:read");
    const canDelete = (await getMyPermissions()).includes("comment:delete");
    return <BlogCommentsClient canDelete={canDelete} />;
}

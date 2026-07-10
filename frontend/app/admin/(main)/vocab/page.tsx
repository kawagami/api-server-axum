import VocabAdminClient from "./vocab-admin-client";
import type { Metadata } from "next";
import { requirePermission, getMyPermissions } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "單字題庫",
    description: "單字闖關題庫管理",
};

export default async function VocabAdminPage() {
    await requirePermission("vocab:read");
    const canUpdate = (await getMyPermissions()).includes("vocab:update");
    return <VocabAdminClient canUpdate={canUpdate} />;
}

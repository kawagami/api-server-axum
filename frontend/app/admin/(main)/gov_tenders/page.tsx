import GovTendersClient from "./gov-tenders-client";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "政府標案",
    description: "政府電子採購網標案追蹤",
};

export default async function GovTendersPage() {
    await requirePermission("gov_tender:read");
    return <GovTendersClient />;
}

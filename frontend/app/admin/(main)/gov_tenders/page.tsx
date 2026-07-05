import GovTendersClient from "./gov-tenders-client";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "政府標案",
    description: "政府電子採購網標案追蹤",
};

export default function GovTendersPage() {
    return <GovTendersClient />;
}

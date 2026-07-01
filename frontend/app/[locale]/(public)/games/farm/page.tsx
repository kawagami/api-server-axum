import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import FarmGame from "./FarmGame";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Farm");
    return { title: t("title"), description: t("description") };
}

export default function FarmPage() {
    return <FarmGame />;
}

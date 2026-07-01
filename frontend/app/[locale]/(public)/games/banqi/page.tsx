import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import BanqiGame from "./BanqiGame";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Banqi");
    return { title: t("title"), description: t("description") };
}

export default function BanqiPage() {
    return <BanqiGame />;
}

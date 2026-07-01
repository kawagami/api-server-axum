import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import AvalonGame from "./AvalonGame";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Avalon");
    return { title: t("title"), description: t("description") };
}

export default function AvalonPage() {
    return <AvalonGame />;
}

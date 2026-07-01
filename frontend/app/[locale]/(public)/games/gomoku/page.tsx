import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import GomokuGame from "./GomokuGame";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Gomoku");
    return { title: t("title"), description: t("description") };
}

export default function GomokuPage() {
    return <GomokuGame />;
}

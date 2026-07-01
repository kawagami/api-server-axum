import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import ChessGame from "./ChessGame";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Chess");
    return { title: t("title"), description: t("description") };
}

export default function ChessPage() {
    return <ChessGame />;
}

import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import WesternChessGame from "./WesternChessGame";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("WesternChess");
    return { title: t("title"), description: t("description") };
}

export default function WesternChessPage() {
    return <WesternChessGame />;
}

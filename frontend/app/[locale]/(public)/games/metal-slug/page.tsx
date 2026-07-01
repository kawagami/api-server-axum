import MetalSlugGame from "@/components/games/metal-slug-game";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Games");
    return { title: t("metalSlugTitle"), description: t("metalSlugDescription") };
}

export default function MetalSlugPage() {
    return <MetalSlugGame />;
}

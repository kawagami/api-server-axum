import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import GoGame from "./GoGame";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Go");
    return { title: t("title"), description: t("description") };
}

export default function GoPage() {
    return <GoGame />;
}

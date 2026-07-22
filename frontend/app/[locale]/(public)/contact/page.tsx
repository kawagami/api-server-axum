import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";
import ContactForm from "./contact-form";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Contact");
    return { title: t("title"), description: t("subtitle") };
}

export default async function ContactPage() {
    // instance 功能開關:message 關閉時整頁不存在(與後端 API 404 一致)
    const settings = await getPublicSettings();
    const enabled = resolveEnabledFeatures(settings.enabled_features);
    if (!isFeatureEnabled(enabled, "message")) notFound();

    const t = await getTranslations("Contact");

    return (
        <div className="w-full max-w-xl px-4 py-4 flex flex-col gap-6">
            <div className="text-center flex flex-col gap-2">
                <h1 className="text-2xl font-bold text-neutral-800 dark:text-neutral-100">{t("title")}</h1>
                <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("subtitle")}</p>
            </div>
            <ContactForm />
        </div>
    );
}

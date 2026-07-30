import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";
import ContactForm from "./contact-form";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

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
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t("title")} description={t("subtitle")} />
            <ContactForm />
        </PageShell>
    );
}

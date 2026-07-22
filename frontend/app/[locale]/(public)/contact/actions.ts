"use server";

import { postContactMessage } from "@/api/contact";

export interface ContactFormState {
    status: "success" | "error" | null;
    // i18n key(對應 messages 的 Contact.result.*),由 client 端翻譯,兼容多語系
    messageKey: "success" | "empty" | "rateLimit" | "invalid" | "failed" | null;
}

export async function submitMessageAction(
    _prevState: ContactFormState,
    formData: FormData,
): Promise<ContactFormState> {
    const content = ((formData.get("content") as string) ?? "").trim();
    const name = ((formData.get("name") as string) ?? "").trim();
    const email = ((formData.get("email") as string) ?? "").trim();

    if (!content) {
        return { status: "error", messageKey: "empty" };
    }

    try {
        await postContactMessage({
            content,
            name: name || undefined,
            email: email || undefined,
        });
        return { status: "success", messageKey: "success" };
    } catch (e) {
        const msg = e instanceof Error ? e.message : "";
        if (msg.includes("429")) return { status: "error", messageKey: "rateLimit" };
        if (msg.includes("422")) return { status: "error", messageKey: "invalid" };
        return { status: "error", messageKey: "failed" };
    }
}

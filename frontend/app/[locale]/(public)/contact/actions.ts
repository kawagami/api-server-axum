"use server";

import { postContactMessage } from "@/api/contact";
import { apiErrorStatus } from "@/libs/api-error";

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
        // 用真正的狀態碼判斷。原本是比對錯誤訊息字串（`msg.includes("429")`），
        // 因為當時 fetchApi 不附 status —— 那種寫法在訊息改文案的當下就靜默失效
        switch (apiErrorStatus(e)) {
            case 429:
                return { status: "error", messageKey: "rateLimit" };
            case 422:
                return { status: "error", messageKey: "invalid" };
            default:
                return { status: "error", messageKey: "failed" };
        }
    }
}

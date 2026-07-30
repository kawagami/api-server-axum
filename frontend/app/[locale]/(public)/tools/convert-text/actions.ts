"use server";

import { postConvertText } from "@/api/tools";

// messageKey 對應 messages 的 ConvertText.*；翻譯留給 client
// （server action 這裡拿不到使用者當前語系，寫死中文就會讓 en / zh-CN 破功）
export interface ConvertTextState {
    status: "success" | "error" | null;
    messageKey: "emptyInput" | "success" | "failed" | null;
    converted_text: string;
}

export async function convertTextAction(prevState: ConvertTextState, formData: FormData): Promise<ConvertTextState> {
    const inputText = formData.get('inputText') as string;
    const direction = formData.get('direction') as "t2s" | "s2t";

    if (!inputText?.trim()) {
        return { status: 'error', messageKey: 'emptyInput', converted_text: '' };
    }

    try {
        const result = await postConvertText(inputText, direction);
        return { status: 'success', messageKey: 'success', converted_text: result.converted_text };
    } catch {
        return { status: 'error', messageKey: 'failed', converted_text: '' };
    }
}

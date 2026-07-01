"use server";

import { postConvertText } from "@/api/tools";

interface ConvertTextState {
    status: string | null;
    message: string | null;
    converted_text: string;
}

export async function convertTextAction(prevState: ConvertTextState, formData: FormData): Promise<ConvertTextState> {
    const inputText = formData.get('inputText') as string;
    const direction = formData.get('direction') as "t2s" | "s2t";

    if (!inputText?.trim()) {
        return { status: 'error', message: '請輸入文本', converted_text: '' };
    }

    try {
        const result = await postConvertText(inputText, direction);
        return { status: 'success', message: '轉換成功！', converted_text: result.converted_text };
    } catch {
        return { status: 'error', message: '轉換失敗，請稍後再試', converted_text: '' };
    }
}

"use server";

import { postConvertText } from "@/api/tools";
import { apiErrorStatus } from "@/libs/api-error";
import { CONVERT_TEXT_MAX_BYTES, utf8ByteLength, type ConversionDirection } from "@/libs/convert-text";

// messageKey 對應 messages 的 ConvertText.*；翻譯留給 client
// （server action 這裡拿不到使用者當前語系，寫死中文就會讓 en / zh-CN 破功）
export type ConvertTextMessageKey = "emptyInput" | "success" | "failed" | "tooLong" | "rateLimited";

export interface ConvertTextState {
    status: "success" | "error" | null;
    messageKey: ConvertTextMessageKey | null;
    converted_text: string;
}

export async function convertTextAction(prevState: ConvertTextState, formData: FormData): Promise<ConvertTextState> {
    const inputText = formData.get('inputText') as string;
    const direction = formData.get('direction') as ConversionDirection;

    // 失敗一律沿用上一次的結果。清成空字串等於「按第二次失敗就把第一次的產物弄丟」，
    // 而使用者當下最可能想做的事就是把它複製走。
    const keep = prevState.converted_text;

    if (!inputText?.trim()) {
        return { status: 'error', messageKey: 'emptyInput', converted_text: keep };
    }

    // 前端已擋一次（送出鈕會 disabled），這裡是第二道：formData 是使用者可構造的輸入，
    // 擋在這裡就省掉一趟對後端的請求與它的 tools rate limit 額度。
    if (utf8ByteLength(inputText) > CONVERT_TEXT_MAX_BYTES) {
        return { status: 'error', messageKey: 'tooLong', converted_text: keep };
    }

    try {
        const result = await postConvertText(inputText, direction);
        return { status: 'success', messageKey: 'success', converted_text: result.converted_text };
    } catch (e) {
        // 依狀態碼分流：422 = 超上限、429 = tools rate limit（每分鐘 20 發）。
        // 全部收斂成一句「請稍後再試」時，使用者對「貼太長」與「按太快」都拿不到可行動資訊；
        // status 為 undefined（網路中斷 / fetchApi 的 10s 逾時）才落到 failed。
        const status = apiErrorStatus(e);
        const messageKey: ConvertTextMessageKey =
            status === 422 ? 'tooLong' : status === 429 ? 'rateLimited' : 'failed';
        return { status: 'error', messageKey, converted_text: keep };
    }
}

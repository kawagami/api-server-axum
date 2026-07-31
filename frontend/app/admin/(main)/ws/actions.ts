"use server";

import adminRequest from "@/libs/adminRequest";

export interface SaySomethingResult {
    ok: boolean;
    message?: string;
}

// createAuthRequest 失敗時丟的 Error 會掛上 status 與後端 ErrorResponse body
interface ApiError extends Error {
    status?: number;
    errorData?: { message?: string } | null;
}

export async function saySomethingToSomeone(
    _prevState: SaySomethingResult,
    formData: FormData,
): Promise<SaySomethingResult> {
    const addr = ((formData.get("addr") as string | null) ?? "").trim();
    const message = ((formData.get("message") as string | null) ?? "").trim();

    if (!addr) {
        return { ok: false, message: "請先從上方連線列表選一個目標" };
    }
    if (!message) {
        return { ok: false, message: "訊息內容不可為空" };
    }

    try {
        await adminRequest({
            url: `${process.env.API_URL}/ws/messages`,
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ addr, message }),
        });
        return { ok: true, message: "已送出" };
    } catch (err) {
        // 後端對「位址格式錯 400 / 連線不存在 404 / 送出失敗 500」都回非 2xx，
        // 這裡才不會把失敗顯示成成功
        const e = err as ApiError;
        if (e.status === 404) {
            return { ok: false, message: "該連線已不存在（可能剛斷線），請重新整理列表" };
        }
        return { ok: false, message: e.errorData?.message ?? e.message };
    }
}

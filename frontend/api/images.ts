"use server";

import adminRequest from "@/libs/adminRequest";
import { runAction, type ActionResult } from "@/libs/api-error";
import type { Image } from "@/types";

export async function getImages(): Promise<Image[]> {
    return adminRequest<Image[]>({
        url: `${process.env.API_URL}/admin/images`,
    });
}

// 上傳 / 刪除都由 client 直接呼叫，回結果而不 throw：後端 400（「不是有效的圖片」「圖片像素過大」）
// 與 413 的狀態碼 / 原因要帶過 Server Action 邊界，throw 的話 production 會被剝掉
export async function uploadImage(formData: FormData): Promise<ActionResult<Image>> {
    return runAction(() => adminRequest<Image>({
        url: `${process.env.API_URL}/admin/images`,
        method: 'POST',
        body: formData,
    }));
}

export async function deleteImage(id: number): Promise<ActionResult> {
    return runAction(async () => {
        await adminRequest({
            url: `${process.env.API_URL}/admin/images/${id}`,
            method: 'DELETE',
        });
    });
}

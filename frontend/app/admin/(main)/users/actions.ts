"use server";

import { revalidatePath } from "next/cache";
import adminRequest from "@/libs/adminRequest";
import { toActionFailure, type ActionResult } from "@/libs/api-error";
import type { User } from "@/types";

// 兩支都由 client 元件直接呼叫，所以回結果而不 throw：
// throw 的話 production 會把 status 與後端訊息剝掉（見 libs/api-error.ts 的 ActionFailure），
// 使用者只看得到通用錯誤，不知道是重名、密碼不合規則還是缺 role:assign。

export async function createUser(input: {
    name: string;
    email?: string;
    password: string;
    role_ids: number[];
}): Promise<ActionResult<User>> {
    // email 選填：空字串就不送（後端當 NULL）
    const body = {
        name: input.name,
        password: input.password,
        role_ids: input.role_ids,
        ...(input.email ? { email: input.email } : {}),
    };
    try {
        const user = await adminRequest<User>({
            url: `${process.env.API_URL}/admin/users`,
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(body),
        });
        revalidatePath("/admin/users");
        return { ok: true, data: user };
    } catch (e) {
        return toActionFailure(e);
    }
}

export async function deleteUser(id: number): Promise<ActionResult> {
    try {
        await adminRequest<void>({
            url: `${process.env.API_URL}/admin/users/${id}`,
            method: "DELETE",
        });
        revalidatePath("/admin/users");
        return { ok: true, data: undefined };
    } catch (e) {
        return toActionFailure(e);
    }
}

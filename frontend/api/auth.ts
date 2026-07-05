"use server";

import adminRequest from "@/libs/adminRequest";
import type { AuthUser } from "@/types";

interface ChangePasswordBody {
    current_password: string;
    new_password: string;
}

/** 目前登入管理員的 email 與 permissions（super_admin 會回傳全部權限） */
export async function getMe(): Promise<AuthUser> {
    return adminRequest<AuthUser>({ url: `${process.env.API_URL}/admin/auth/me` });
}

export async function postChangePassword(body: ChangePasswordBody): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/auth/change_password`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
}

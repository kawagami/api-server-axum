"use server";

import adminRequest from "@/libs/adminRequest";
import { runAction, type ActionResult } from "@/libs/api-error";
import type { AuthUser, PasskeyItem } from "@/types";
import type {
    PublicKeyCredentialCreationOptionsJSON,
    RegistrationResponseJSON,
} from "@simplewebauthn/browser";

interface ChangePasswordBody {
    current_password: string;
    new_password: string;
}

/** 目前登入管理員的 email 與 permissions（super_admin 會回傳全部權限） */
export async function getMe(): Promise<AuthUser> {
    return adminRequest<AuthUser>({ url: `${process.env.API_URL}/admin/auth/me` });
}

/** 回結果而不 throw：client 要靠 status 分辨「目前密碼錯誤（422）」，throw 的話 production 會被剝掉 */
export async function postChangePassword(body: ChangePasswordBody): Promise<ActionResult> {
    return runAction(async () => {
        await adminRequest<void>({
            url: `${process.env.API_URL}/admin/auth/change_password`,
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        });
    });
}

/** passkey 註冊挑戰；ceremony 本體在瀏覽器跑，前端取回傳的 publicKey 內層餵 startRegistration */
export async function beginPasskeyRegistration(): Promise<{ publicKey: PublicKeyCredentialCreationOptionsJSON }> {
    return adminRequest({
        url: `${process.env.API_URL}/admin/auth/passkeys/register/begin`,
        method: 'POST',
    });
}

/** 回結果而不 throw：client 要靠 status 分辨「此 passkey 已註冊過（409）」 */
export async function finishPasskeyRegistration(
    credential: RegistrationResponseJSON,
    label: string,
): Promise<ActionResult> {
    return runAction(async () => {
        await adminRequest<void>({
            url: `${process.env.API_URL}/admin/auth/passkeys/register/finish`,
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ label, credential }),
        });
    });
}

export async function getPasskeys(): Promise<PasskeyItem[]> {
    return adminRequest<PasskeyItem[]>({ url: `${process.env.API_URL}/admin/auth/passkeys` });
}

export async function deletePasskey(id: number): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/auth/passkeys/${id}`,
        method: 'DELETE',
    });
}

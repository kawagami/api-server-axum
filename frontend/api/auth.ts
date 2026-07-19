"use server";

import adminRequest from "@/libs/adminRequest";
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

export async function postChangePassword(body: ChangePasswordBody): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/auth/change_password`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
}

/** passkey 註冊挑戰；ceremony 本體在瀏覽器跑，前端取回傳的 publicKey 內層餵 startRegistration */
export async function beginPasskeyRegistration(): Promise<{ publicKey: PublicKeyCredentialCreationOptionsJSON }> {
    return adminRequest({
        url: `${process.env.API_URL}/admin/auth/passkeys/register/begin`,
        method: 'POST',
    });
}

export async function finishPasskeyRegistration(
    credential: RegistrationResponseJSON,
    label: string,
): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/auth/passkeys/register/finish`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ label, credential }),
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

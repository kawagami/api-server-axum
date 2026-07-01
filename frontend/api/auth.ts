"use server";

import adminRequest from "@/libs/adminRequest";

interface ChangePasswordBody {
    current_password: string;
    new_password: string;
}

export async function postChangePassword(body: ChangePasswordBody): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/auth/change_password`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
}

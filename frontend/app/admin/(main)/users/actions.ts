"use server";

import { revalidatePath } from "next/cache";
import adminRequest from "@/libs/adminRequest";

export async function createUser(input: {
    name: string;
    email?: string;
    password: string;
    role_ids: number[];
}): Promise<void> {
    // email 選填：空字串就不送（後端當 NULL）
    const body = {
        name: input.name,
        password: input.password,
        role_ids: input.role_ids,
        ...(input.email ? { email: input.email } : {}),
    };
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/users`,
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
    });
    revalidatePath("/admin/users");
}

export async function deleteUser(user: { id: string; name: string }): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/users`,
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: Number(user.id), name: user.name }),
    });
    revalidatePath("/admin/users");
}

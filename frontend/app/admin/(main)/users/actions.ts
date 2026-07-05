"use server";

import { revalidatePath } from "next/cache";
import adminRequest from "@/libs/adminRequest";

export async function createUser(input: {
    name: string;
    email: string;
    password: string;
    role_ids: number[];
}): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/users`,
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(input),
    });
    revalidatePath("/admin/users");
}

export async function deleteUser(user: { id: string; name: string; email: string }): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/users`,
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: Number(user.id), name: user.name, email: user.email }),
    });
    revalidatePath("/admin/users");
}

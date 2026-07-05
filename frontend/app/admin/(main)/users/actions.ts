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

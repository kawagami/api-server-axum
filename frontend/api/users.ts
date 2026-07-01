"use server";

import adminRequest from "@/libs/adminRequest";
import type { User, Role } from "@/types";

export async function getUsers(): Promise<User[]> {
    return adminRequest<User[]>({
        url: `${process.env.API_URL}/admin/users`,
    });
}

export async function getUserRoles(userId: string): Promise<Role[]> {
    return adminRequest<Role[]>({ url: `${process.env.API_URL}/admin/users/${userId}/roles` });
}

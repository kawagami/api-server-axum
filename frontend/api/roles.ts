"use server";

import adminRequest from "@/libs/adminRequest";
import type { Role, Permission } from "@/types";

export async function getRoles(): Promise<Role[]> {
    return adminRequest<Role[]>({ url: `${process.env.API_URL}/admin/roles` });
}

export async function getPermissions(): Promise<Permission[]> {
    return adminRequest<Permission[]>({ url: `${process.env.API_URL}/admin/permissions` });
}

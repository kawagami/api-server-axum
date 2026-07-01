"use server";

import { revalidatePath } from 'next/cache';
import adminRequest from "@/libs/adminRequest";
import type { Role } from "@/types";

export async function createRole(formData: FormData): Promise<void> {
    await adminRequest<Role>({
        url: `${process.env.API_URL}/admin/roles`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            name: formData.get('name'),
            description: formData.get('description') || undefined,
        }),
    });
    revalidatePath('/admin/roles');
}

export async function deleteRole(id: number): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/roles/${id}`,
        method: 'DELETE',
    });
    revalidatePath('/admin/roles');
}

export async function setRolePermissions(roleId: number, permissionIds: number[]): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/roles/${roleId}/permissions`,
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ permission_ids: permissionIds }),
    });
    revalidatePath('/admin/roles');
}

export async function setUserRoles(userId: string, roleIds: number[]): Promise<void> {
    await adminRequest<void>({
        url: `${process.env.API_URL}/admin/users/${userId}/roles`,
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ role_ids: roleIds }),
    });
    revalidatePath('/admin/users');
}

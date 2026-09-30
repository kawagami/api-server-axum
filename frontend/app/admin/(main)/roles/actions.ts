"use server";

import { revalidatePath } from 'next/cache';
import adminRequest from "@/libs/adminRequest";
import { runAction, type ActionResult } from "@/libs/api-error";
import type { Role } from "@/types";

// 全部由 client 元件直接呼叫，回結果而不 throw（理由同 users/actions.ts）：
// 權限放大防護（授出超過自己的權限、指派 super_admin、改自己的角色）回的 403
// 帶著原因，throw 出去在 production 就只剩通用錯誤。

export async function createRole(formData: FormData): Promise<ActionResult<Role>> {
    return runAction(async () => {
        const role = await adminRequest<Role>({
            url: `${process.env.API_URL}/admin/roles`,
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                name: formData.get('name'),
                description: formData.get('description') || undefined,
            }),
        });
        revalidatePath('/admin/roles');
        return role;
    });
}

export async function deleteRole(id: number): Promise<ActionResult> {
    return runAction(async () => {
        await adminRequest<void>({
            url: `${process.env.API_URL}/admin/roles/${id}`,
            method: 'DELETE',
        });
        revalidatePath('/admin/roles');
    });
}

export async function setRolePermissions(roleId: number, permissionIds: number[]): Promise<ActionResult> {
    return runAction(async () => {
        await adminRequest<void>({
            url: `${process.env.API_URL}/admin/roles/${roleId}/permissions`,
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ permission_ids: permissionIds }),
        });
        revalidatePath('/admin/roles');
    });
}

export async function setUserRoles(userId: number, roleIds: number[]): Promise<ActionResult> {
    return runAction(async () => {
        await adminRequest<void>({
            url: `${process.env.API_URL}/admin/users/${userId}/roles`,
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ role_ids: roleIds }),
        });
        revalidatePath('/admin/users');
    });
}

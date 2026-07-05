import { cache } from "react";
import { redirect } from "next/navigation";
import { getMe } from "@/api/auth";
import type { AuthUser } from "@/types";

/**
 * 目前登入管理員（email + permissions，super_admin 含全部）。
 * 以 React cache() 去重：同一次請求內 layout + 各頁 guard/頁面只實際打一次 /admin/auth/me。
 */
export const getCurrentAdmin = cache(async (): Promise<AuthUser> => getMe());

/** 目前登入管理員的 permissions。 */
export async function getMyPermissions(): Promise<string[]> {
    return (await getCurrentAdmin()).permissions;
}

/** 頁面 guard：缺對應權限就導回後台首頁（避免直接打 URL 繞過選單過濾）。 */
export async function requirePermission(permission: string): Promise<void> {
    const permissions = await getMyPermissions();
    if (!permissions.includes(permission)) {
        redirect("/admin");
    }
}

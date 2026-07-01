"use server";

import adminRequest from "@/libs/adminRequest";
import type { VisitorStats } from "@/types";

// 到訪統計：需登入後台帳號的 stat:read 權限
export async function getVisitorStats(days: number): Promise<VisitorStats> {
    return adminRequest<VisitorStats>({
        url: `${process.env.API_URL}/admin/stats/visitors?days=${days}`,
    });
}

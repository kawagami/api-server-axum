"use server";

import adminRequest from "@/libs/adminRequest";
import type { SystemMetric } from "@/types";

// 系統指標：需登入後台帳號的 metric:read 權限；hours 範圍 1~168，預設 24
export async function getSystemMetrics(hours = 24): Promise<SystemMetric[]> {
    return adminRequest<SystemMetric[]>({
        url: `${process.env.API_URL}/metrics?hours=${hours}`,
    });
}

"use client";

import { useEffect } from "react";
import { startTokenRefresh } from "@/libs/token-refresh";

// (main) layout 在 proxy.ts 保護下才會渲染，直接啟動刷新即可；
// session 過期時 doRefresh 收 401 自行導回 /admin/login
export default function TokenRefreshInit() {
    useEffect(() => {
        startTokenRefresh();
    }, []);

    return null;
}

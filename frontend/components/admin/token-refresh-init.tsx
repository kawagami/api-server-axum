"use client";

import { useEffect } from "react";
import { restartTokenRefresh } from "@/libs/token-refresh";

export default function TokenRefreshInit() {
    useEffect(() => {
        restartTokenRefresh();
    }, []);

    return null;
}

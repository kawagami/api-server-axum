import type { MetadataRoute } from "next";

export default function robots(): MetadataRoute.Robots {
    return {
        rules: {
            userAgent: "*",
            allow: "/",
            // 後台、API、OAuth callback 與需登入的會員頁沒有索引價值
            disallow: [
                "/admin",
                "/api/",
                "/auth/",
                "/*/dashboard",
                "/*/profile",
                "/*/invoices",
                "/*/lotto",
                "/*/ledger",
                "/*/portfolio",
                "/*/login",
            ],
        },
        sitemap: "https://kawa.homes/sitemap.xml",
    };
}

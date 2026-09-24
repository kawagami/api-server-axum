// ⚠️ 刻意**不是** "use server"：那會把這支登記成 Server Action（有 action ID、server 會受理呼叫），
// 而它收的是**完整 URL** 並自動帶上 session token —— 哪天被 client 元件 import，就等於一個
// 免登入也能用的 SSRF 代理（打得到內網 backend:3000 / database 並把回應帶回去）。
// 它只該被 server 端（api/*.ts 的 server action、Route Handler）呼叫；`server-only` 讓誤用直接 build 失敗。
import "server-only";

import { getLocale } from 'next-intl/server';
import { redirect } from 'next/navigation';
import { createAuthRequest } from "@/libs/createAuthRequest";

const memberRequest = createAuthRequest("access_token", async () => {
    const locale = await getLocale();
    redirect(`/${locale}/login`);
});

export default memberRequest;

import { headers } from "next/headers";

/**
 * 把訪客真實 IP 轉發給後端。
 *
 * 為什麼需要：後端 `middleware/rate_limit.rs` 在 `TRUST_CF_HEADER=true`（生產值）時
 * 以 `CF-Connecting-IP` 當限流 key，缺這個 header 就 fallback 到 socket IP。而所有
 * 「Next server 代打後端」的請求 socket IP 都是 frontend 容器的內網位址 —— 於是
 * **所有訪客共用同一個 bucket**：任何人打 5 次 admin 登入就能讓真正的管理員在該分鐘內
 * 登不進去，訪客留言（5/min）與 blog 留言（10/min）同樣可被單一來源鎖死全站。
 *
 * nginx 在 server 層已把這個 header 覆寫成真實來源（見 deploy/nginx/conf.d/*.conf 的
 * `proxy_set_header CF-Connecting-IP $remote_addr`，而 $remote_addr 只對 Cloudflare
 * 網段被還原），所以這裡讀到的值不可由客戶端偽造。
 *
 * ⚠️ **不要把這個放進 `libs/fetchApi.ts` 內部**：`api/blogs.ts` 與 `api/settings.ts`
 * 走 `next: { revalidate, tags }` 的 Data Cache，加上逐訪客變動的 header 會讓快取
 * 按 IP 分裂（等於沒有快取）。所以設計成 opt-in，只用在實際受限流、且本來就
 * `cache: 'no-store'` 的呼叫點。
 *
 * header 缺失時回空物件 —— 行為與修正前相同（後端 fallback socket IP），不會更糟。
 */
export async function clientIpHeaders(): Promise<Record<string, string>> {
    const ip = (await headers()).get("CF-Connecting-IP");
    return ip ? { "CF-Connecting-IP": ip } : {};
}

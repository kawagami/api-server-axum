import type { MetadataRoute } from "next";
import { routing } from "@/i18n/routing";
import { getBlogs } from "@/api/blogs";
import { getPublicSettings } from "@/api/settings";
import { getChangelogRepo } from "@/api/github";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";
import { TOOLS, GAMES } from "@/libs/site-nav";

/** = 後端 MAX_PER_PAGE */
const SITEMAP_BLOG_PAGE_SIZE = 200;
/** 翻頁保險絲：total 異常時不至於無限打後端（200 × 50 = 10,000 篇） */
const SITEMAP_MAX_BLOG_PAGES = 50;

const BASE = "https://kawa.homes";

// 需登入的會員頁（portfolio / dashboard / profile）不進 sitemap
const CORE_PATHS: { path: string; feature?: string; priority: number }[] = [
    { path: "", priority: 1 },
    { path: "/blogs", feature: "blog", priority: 0.9 },
    { path: "/about", priority: 0.6 },
    { path: "/tools", priority: 0.7 },
    { path: "/games", priority: 0.7 },
    { path: "/vocab", feature: "vocab", priority: 0.6 },
    { path: "/contact", feature: "message", priority: 0.4 },
];

/** 一個路徑 × 三語系，彼此互為 alternates（hreflang 也一併給搜尋引擎） */
function entry(path: string, priority: number, lastModified?: string | Date): MetadataRoute.Sitemap {
    return routing.locales.map((locale) => ({
        url: `${BASE}/${locale}${path}`,
        lastModified,
        priority,
        alternates: {
            languages: Object.fromEntries(
                routing.locales.map((l) => [l, `${BASE}/${l}${path}`]),
            ),
        },
    }));
}

export const revalidate = 3600;

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
    // instance 功能開關關掉的頁面會 404，不該出現在 sitemap
    const enabled = await getPublicSettings()
        .then((s) => resolveEnabledFeatures(s.enabled_features))
        .catch(() => null);

    const staticPaths = [
        ...CORE_PATHS,
        ...TOOLS.map((t) => ({ path: t.href, feature: t.feature, priority: 0.5 })),
        ...GAMES.map((g) => ({ path: g.href, feature: g.feature, priority: 0.5 })),
    ].filter(({ feature }) => isFeatureEnabled(enabled, feature));

    const entries = staticPaths.flatMap(({ path, priority }) => entry(path, priority));

    // /changelog 的開關是 GITHUB_REPO（不是 enabled_features），沒設 repo 時整頁 404
    if (await getChangelogRepo()) entries.push(...entry("/changelog", 0.5));

    // 文章清單：API 掛掉時只是少了動態項目，不該讓整份 sitemap（與 build）失敗
    if (isFeatureEnabled(enabled, "blog")) {
        try {
            // 後端 per_page 上限 200（structs/pagination.rs 的 MAX_PER_PAGE），要多少就得翻頁 ——
            // 以前一次要 500，被夾成 200 後第 201 篇起就靜默從 sitemap 消失。
            // 中途失敗時已抓到的頁照樣收進去
            for (let page = 1, seen = 0; page <= SITEMAP_MAX_BLOG_PAGES; page++) {
                const { data: blogs, total } = await getBlogs({ page, per_page: SITEMAP_BLOG_PAGE_SIZE });
                entries.push(
                    ...blogs.flatMap((blog) =>
                        entry(`/blogs/${blog.id}`, 0.8, blog.updated_at ?? blog.created_at ?? undefined),
                    ),
                );
                seen += blogs.length;
                if (blogs.length < SITEMAP_BLOG_PAGE_SIZE || seen >= total) break;
            }
        } catch {
            // 靜默略過：靜態路徑仍會產出
        }
    }

    return entries;
}

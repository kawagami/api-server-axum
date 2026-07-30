import { routing } from "@/i18n/routing";

export const SITE_NAME = "Kawa's Homes";

/**
 * 多語系頁面的 canonical + hreflang 單一來源。
 * 三個語系是同一份內容的不同語言版本，沒有 alternates.languages 的話搜尋引擎會當成重複內容。
 * x-default 指向預設語系（routing.defaultLocale）。
 *
 * @param locale 當前語系
 * @param path   locale 之後的路徑，開頭要有 `/`；首頁傳空字串
 */
export function localeAlternates(locale: string, path: string = "") {
    const languages = Object.fromEntries(
        routing.locales.map((l) => [l, `/${l}${path}`]),
    ) as Record<string, string>;

    return {
        canonical: `/${locale}${path}`,
        languages: {
            ...languages,
            "x-default": `/${routing.defaultLocale}${path}`,
        },
    };
}

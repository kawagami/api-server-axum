import { defineRouting } from 'next-intl/routing';

export const routing = defineRouting({
    locales: ['zh-TW', 'zh-CN', 'en'],
    defaultLocale: 'zh-TW',
    // 顯式寫出（本來就是預設值）：proxy.ts 的 memberPaths 是拿 `/${locale}${path}` 比對，
    // 依賴「每個路徑都一定帶 locale prefix」。改成 'as-needed' 會讓無 prefix 的
    // /dashboard 之類繞過整段 member 保護，而且不會有任何錯誤訊息。
    localePrefix: 'always',
});

import { defineRouting } from 'next-intl/routing';

export const routing = defineRouting({
    locales: ['zh-TW', 'zh-CN', 'en'],
    defaultLocale: 'zh-TW',
});

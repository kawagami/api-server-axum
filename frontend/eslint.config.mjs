import coreWebVitals from 'eslint-config-next/core-web-vitals';

// 專案慣例改由 lint 守（2026-09-25）。原本只寫在 ARCHITECTURE.md，漏用沒有任何執行期徵兆。
// 管不到的（Tailwind class、檔案層級的規則）在 scripts/check-conventions.sh，CI 兩者都跑。

/** 公開頁：導航一律走 locale-aware 的 `@/i18n/navigation`，否則換頁會掉 locale prefix */
const publicNavigation = {
    files: ['app/\\[locale\\]/**/*.{ts,tsx}', 'components/*.{ts,tsx}'],
    rules: {
        'no-restricted-imports': ['error', {
            paths: [
                {
                    name: 'next/link',
                    message: '公開頁改用 `@/i18n/navigation` 的 Link（自動帶 locale）。外部連結確實要用 next/link 時，以 eslint-disable-next-line 註明「外部連結」。',
                },
                {
                    name: 'next/navigation',
                    importNames: ['useRouter', 'redirect', 'permanentRedirect', 'usePathname'],
                    message: '公開頁改用 `@/i18n/navigation` 的同名匯出（自動帶 locale）。',
                },
            ],
        }],
    },
};

/** Server Action 的寫入路徑：revalidateTag 不會失效瀏覽器 Router Cache（見 ARCHITECTURE.md「Blog 注意事項」） */
const serverActionCache = {
    files: ['api/**/*.ts', 'actions/**/*.ts', 'app/**/actions.ts'],
    rules: {
        'no-restricted-imports': ['error', {
            paths: [{
                name: 'next/cache',
                importNames: ['revalidateTag'],
                message: 'Server Action 裡改用 `updateTag(tag)`：revalidateTag 帶非 expire:0 的 profile 時不失效 Router Cache，存檔後重進頁面會看到舊內容。',
            }],
        }],
    },
};

/** 全站 */
const global = {
    rules: {
        'no-restricted-syntax': ['error',
            {
                selector: "CallExpression[callee.name='alert'], CallExpression[callee.object.name='window'][callee.property.name='alert']",
                message: '改用 `components/toast.tsx` 的 useToast()（頁面層級錯誤用 ErrorBanner）。',
            },
            {
                selector: "CallExpression[callee.property.name=/^toLocale(Date|Time)?String$/][callee.object.type='NewExpression'][callee.object.callee.name='Date'][arguments.length=0]",
                message: '日期不帶 locale / timeZone 會隨執行環境變（server 容器是 UTC、client 跟瀏覽器）。後台用 `libs/admin-datetime.ts`，前台用 `Intl.DateTimeFormat(locale, { timeZone: "Asia/Taipei" })`。',
            },
        ],
    },
};

const config = [
    { ignores: ['.next/**', 'node_modules/**', 'public/**'] },
    ...coreWebVitals,
    global,
    publicNavigation,
    serverActionCache,
];

export default config;

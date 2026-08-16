import createNextIntlPlugin from 'next-intl/plugin';

const withNextIntl = createNextIntlPlugin('./i18n/request.ts');

/** @type {import('next').NextConfig} */
const nextConfig = {
    output: 'standalone',
    // sharp 0.35 起，libvips 的 .so 無法被 standalone 的 output tracing 靜態偵測到，
    // 產出裡只會有 @img/sharp-libvips-* 的 package.json 與 lib/index.js、缺 libvips-cpp.so。
    // 後果非常隱蔽：Next 的 image-optimizer 對最佳化失敗是「靜默 fallback 回原圖」
    // （catch → return upstreamBuffer），不報錯、不改狀態碼，只是每張圖都以原始尺寸送出。
    // 2026-07-25 實際踩過：/_next/image?w=64 回傳的位元組數與原圖完全相同。
    // 驗證方式：對同一張圖用不同 w 打 /_next/image，size 必須隨 w 變化。
    // glob 用 linux* 同時涵蓋 glibc（本機 linux-x64）與 musl（Docker 的 linuxmusl-x64）。
    // @swc/helpers 的 esm/ 同樣追蹤不到（next 16.3.1 起，2026-08-16 踩過，全站 521/522）。
    // SWC 編譯產物在 runtime 會 require '@swc/helpers/esm/_interop_require_default.js'，
    // 但 tracing 只抄得到 cjs/ —— 產出裡那個套件只剩 cjs/ 與 package.json。
    // 症狀是 standalone server 一啟動就 MODULE_NOT_FOUND 崩潰迴圈，
    // 連帶 nginx 解析不到 frontend upstream 也起不來（host not found in upstream）。
    // ⚠ `next build` 過**不代表**這裡沒問題：build 不會啟動 standalone server。
    // 驗證方式：把 .next/standalone 複製到專案外的目錄（避免 node 往上找到 frontend/node_modules
    // 而掩蓋掉缺件）再 `node server.js`，起得來才算過。
    outputFileTracingIncludes: {
        '/**': [
            './node_modules/.pnpm/@img+sharp-libvips-linux*/**/*',
            // 只指到 esm/，不要用 `@swc+helpers*/**/*` —— 那會掃到同層的
            // node_modules/tslib（指向目錄的 symlink），Turbopack 當檔案讀會 panic：
            // `reading file ".../@swc+helpers@0.5.23/node_modules/tslib" — Is a directory`
            './node_modules/.pnpm/@swc+helpers@*/node_modules/@swc/helpers/esm/**/*',
        ],
    },
    async redirects() {
        return [
            { source: '/convert-text', destination: '/tools/convert-text', permanent: true },
            { source: '/countdown', destination: '/tools/countdown', permanent: true },
            { source: '/new-password', destination: '/tools/new-password', permanent: true },
            { source: '/roster', destination: '/tools/roster', permanent: true },
        ];
    },
    images: {
        remotePatterns: [
            {
                protocol: 'https',
                hostname: 'media.kawa.homes',
                port: '',
                pathname: '/**'
            },
        ]
    },
    reactStrictMode: true,
    experimental: {
        serverActions: {
            // next-blog:3000 是已改名的舊 compose service（現為 frontend），已移除
            allowedOrigins: ["kawa.homes"],
            bodySizeLimit: '10mb'
        }
    }
};

export default withNextIntl(nextConfig);

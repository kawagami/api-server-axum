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
    outputFileTracingIncludes: {
        '/**': ['./node_modules/.pnpm/@img+sharp-libvips-linux*/**/*'],
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

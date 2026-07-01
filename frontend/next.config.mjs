import createNextIntlPlugin from 'next-intl/plugin';

const withNextIntl = createNextIntlPlugin('./i18n/request.ts');

/** @type {import('next').NextConfig} */
const nextConfig = {
    output: 'standalone',
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
                hostname: 'storage.googleapis.com',
                port: '',
                pathname: '/**'
            },
            {
                protocol: 'https',
                hostname: 'axum.kawa.homes',
                port: '',
                pathname: '/uploads/**'
            },
        ]
    },
    reactStrictMode: true,
    experimental: {
        serverActions: {
            allowedOrigins: ["kawa.homes", "next-blog:3000"],
            bodySizeLimit: '10mb'
        }
    }
};

export default withNextIntl(nextConfig);

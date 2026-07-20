import type { PublicSettings } from '@/api/settings';

// 前端上傳前壓縮設定（來源:後端 app_settings，經 GET /settings/public 下發）。
// quality 為 1–100（與後端一致），compressImage 內部再換算成 canvas 的 0–1。
export interface ImageCompressConfig {
    enabled: boolean;
    quality: number;
    maxEdge: number;
}

// 設定缺失/壞值時的 fallback，與 migration 預設值一致。
export const DEFAULT_IMAGE_COMPRESS: ImageCompressConfig = {
    enabled: true,
    quality: 80,
    maxEdge: 2560,
};

function toInt(value: string | undefined, fallback: number, min: number, max: number): number {
    const n = Number(value);
    if (!Number.isInteger(n) || n < min || n > max) return fallback;
    return n;
}

/** 從公開設定收斂出前端壓縮設定;任何欄位缺失或不合法都退回預設。 */
export function resolveImageCompressConfig(s: PublicSettings): ImageCompressConfig {
    return {
        enabled: s.image_client_compress !== 'false', // 只有明確 "false" 才關閉
        quality: toInt(s.image_client_quality, DEFAULT_IMAGE_COMPRESS.quality, 1, 100),
        maxEdge: toInt(s.image_client_max_edge, DEFAULT_IMAGE_COMPRESS.maxEdge, 64, 16383),
    };
}

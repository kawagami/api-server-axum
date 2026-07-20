import { validateFileSizes, type UploadProgress } from '@/libs/upload-limits';
import { DEFAULT_IMAGE_COMPRESS, type ImageCompressConfig } from '@/libs/image-config';

// 上傳前在瀏覽器端縮圖 + 轉 WebP,大幅縮小上傳流量與後端轉檔負擔。
// 長邊上限與品質由後端設定（image_client_max_edge / image_client_quality）下發,預設 2560px / q80。
// 這只是最佳化,不是信任邊界——後端 process_image 的 decode 驗證/重編照舊,送什麼都會被驗。
// 任何一步失敗都回傳原檔,交給後端處理(行為等同未壓縮的舊流程)。
// 小檔壓縮效益低,跳過以免多一次失真
const SKIP_BYTES = 512 * 1024;

function toBlob(canvas: HTMLCanvasElement, type: string, quality: number): Promise<Blob | null> {
    return new Promise(resolve => canvas.toBlob(resolve, type, quality));
}

/** 單張壓縮；GIF（canvas 只剩第一幀，動畫會壞）與小檔直接跳過。quality 為 1–100。 */
export async function compressImage(
    file: File,
    opts: { quality: number; maxEdge: number } = DEFAULT_IMAGE_COMPRESS,
): Promise<File> {
    if (file.type === 'image/gif' || file.size <= SKIP_BYTES) return file;
    const quality = opts.quality / 100; // canvas.toBlob 用 0–1
    try {
        // createImageBitmap 預設把 EXIF 方向烤進像素,直圖不會躺下
        const bitmap = await createImageBitmap(file);
        const scale = Math.min(1, opts.maxEdge / Math.max(bitmap.width, bitmap.height));
        const w = Math.max(1, Math.round(bitmap.width * scale));
        const h = Math.max(1, Math.round(bitmap.height * scale));
        const canvas = document.createElement('canvas');
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext('2d');
        if (!ctx) return file;
        ctx.drawImage(bitmap, 0, 0, w, h);
        bitmap.close();

        let blob = await toBlob(canvas, 'image/webp', quality);
        if (!blob || blob.type !== 'image/webp') {
            // Safari 的 toBlob 不支援 webp(會默默回 png)。PNG 來源改出 png 保留透明度
            // (仍享縮圖效益),其他出 jpeg(後端會再轉 webp)
            blob = file.type === 'image/png'
                ? await toBlob(canvas, 'image/png', quality)
                : await toBlob(canvas, 'image/jpeg', quality);
        }
        if (!blob || blob.size >= file.size) return file;

        const ext = { 'image/webp': 'webp', 'image/jpeg': 'jpg', 'image/png': 'png' }[blob.type] ?? 'webp';
        const base = file.name.replace(/\.[^.]+$/, '');
        return new File([blob], `${base}.${ext}`, { type: blob.type });
    } catch {
        return file;
    }
}

/**
 * Pipeline 逐張上傳:上傳第 i 張的同時壓縮第 i+1 張(上傳等網路/後端時,瀏覽器
 * 正好做下一張的解碼/編碼),總時間 ≈ max(壓縮總時, 上傳總時) 而非兩者相加。
 * 順序保持(onUploaded 依 index 循序觸發);單檔超限(大 GIF/解不開退回的原檔)
 * 在輪到該張時丟錯中斷,已完成的張數不受影響。
 * config.enabled=false 時跳過壓縮,直接送原檔(後端仍會驗證+轉檔)。
 */
export async function compressAndUploadEach<T>(
    files: File[],
    upload: (file: File) => Promise<T>,
    onProgress: (progress: UploadProgress) => void,
    onUploaded: (result: T, index: number) => void,
    config: ImageCompressConfig = DEFAULT_IMAGE_COMPRESS,
): Promise<void> {
    const prep = (f: File) => config.enabled ? compressImage(f, config) : Promise.resolve(f);
    let pending = prep(files[0]);
    onProgress({ phase: 'compress', current: 1, total: files.length });
    for (let i = 0; i < files.length; i++) {
        const compressed = await pending;
        if (i + 1 < files.length) pending = prep(files[i + 1]);
        const sizeError = validateFileSizes([compressed]);
        if (sizeError) throw Object.assign(new Error(sizeError), { userMessage: sizeError });
        onProgress({ phase: 'upload', current: i + 1, total: files.length });
        onUploaded(await upload(compressed), i);
    }
}

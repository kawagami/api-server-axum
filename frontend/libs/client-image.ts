// 上傳前在瀏覽器端縮圖 + 轉 WebP q80（長邊上限 2560px），大幅縮小上傳流量與後端轉檔負擔。
// 這只是最佳化,不是信任邊界——後端 process_image 的 decode 驗證/重編照舊,送什麼都會被驗。
// 任何一步失敗都回傳原檔,交給後端處理(行為等同未壓縮的舊流程)。
const MAX_LONG_EDGE = 2560;
const QUALITY = 0.8;
// 小檔壓縮效益低,跳過以免多一次失真
const SKIP_BYTES = 512 * 1024;

function toBlob(canvas: HTMLCanvasElement, type: string): Promise<Blob | null> {
    return new Promise(resolve => canvas.toBlob(resolve, type, QUALITY));
}

/** 單張壓縮；GIF（canvas 只剩第一幀，動畫會壞）與小檔直接跳過。 */
export async function compressImage(file: File): Promise<File> {
    if (file.type === 'image/gif' || file.size <= SKIP_BYTES) return file;
    try {
        // createImageBitmap 預設把 EXIF 方向烤進像素,直圖不會躺下
        const bitmap = await createImageBitmap(file);
        const scale = Math.min(1, MAX_LONG_EDGE / Math.max(bitmap.width, bitmap.height));
        const w = Math.max(1, Math.round(bitmap.width * scale));
        const h = Math.max(1, Math.round(bitmap.height * scale));
        const canvas = document.createElement('canvas');
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext('2d');
        if (!ctx) return file;
        ctx.drawImage(bitmap, 0, 0, w, h);
        bitmap.close();

        let blob = await toBlob(canvas, 'image/webp');
        if (!blob || blob.type !== 'image/webp') {
            // Safari 的 toBlob 不支援 webp(會默默回 png)。PNG 來源改出 png 保留透明度
            // (仍享縮圖效益),其他出 jpeg(後端會再轉 webp)
            blob = file.type === 'image/png'
                ? await toBlob(canvas, 'image/png')
                : await toBlob(canvas, 'image/jpeg');
        }
        if (!blob || blob.size >= file.size) return file;

        const ext = { 'image/webp': 'webp', 'image/jpeg': 'jpg', 'image/png': 'png' }[blob.type] ?? 'webp';
        const base = file.name.replace(/\.[^.]+$/, '');
        return new File([blob], `${base}.${ext}`, { type: blob.type });
    } catch {
        return file;
    }
}

/** 逐張壓縮（避免同時解多張大圖吃爆手機記憶體），回報進度。 */
export async function compressImages(
    files: File[],
    onProgress?: (current: number, total: number) => void,
): Promise<File[]> {
    const out: File[] = [];
    for (let i = 0; i < files.length; i++) {
        onProgress?.(i + 1, files.length);
        out.push(await compressImage(files[i]));
    }
    return out;
}

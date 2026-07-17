// 上傳大小限制對齊 server 端最緊的一道：backend RequestBodyLimitLayer 10*1000*1000 bytes
// （nginx client_max_body_size 10M 與 Next server action bodySizeLimit 10mb 都是 10*1024*1024，較寬）。
// 限制的是「整個請求」，多檔超過時前端自動切批連打；預留 multipart 邊界/標頭開銷，取 9.5MB。
export const MAX_UPLOAD_TOTAL_BYTES = 9_500_000;

// 每批張數上限:後端逐張 decode+轉 WebP(1核 VPS 約 1~3 秒/張),張數太多會撞
// adminRequest 的 30 秒逾時;5 張約 10~15 秒,留有餘裕
export const MAX_UPLOAD_FILES_PER_BATCH = 5;

// 瀏覽器端整體逾時:傳輸層卡死(如 QUIC 上傳停滯)時讓請求明確失敗,而非無限 pending
const UPLOAD_TIMEOUT_MS = 90_000;
const TIMEOUT_MARK = 'upload-timeout';

/** 包住單批上傳,逾時丟帶記號的錯誤(底層請求無法取消,但 UI 能脫身重試)。 */
export function withUploadTimeout<T>(promise: Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
            reject(Object.assign(new Error('upload timeout'), { mark: TIMEOUT_MARK }));
        }, UPLOAD_TIMEOUT_MS);
        promise.then(resolve, reject).finally(() => clearTimeout(timer));
    });
}

function formatMB(bytes: number): string {
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** 上傳流程的階段進度（compress = 前端壓縮、upload = 分批上傳）。 */
export type UploadProgress = { phase: 'compress' | 'upload'; current: number; total: number };

/** 上傳鈕的進行中文案。 */
export function uploadProgressLabel(p: UploadProgress | null): string {
    if (!p) return '上傳中...';
    const label = p.phase === 'compress' ? '處理圖片' : '上傳中';
    return p.total > 1 ? `${label} (${p.current}/${p.total})...` : `${label}...`;
}

/** 找出單檔就超過上限的圖片（分批也救不了），回錯誤訊息；沒有回 null。 */
export function validateFileSizes(files: File[]): string | null {
    const over = files.filter(f => f.size > MAX_UPLOAD_TOTAL_BYTES);
    if (!over.length) return null;
    const names = over.slice(0, 3).map(f => `${f.name}（${formatMB(f.size)}）`).join('、');
    const suffix = over.length > 3 ? ` 等 ${over.length} 張` : '';
    return `${names}${suffix} 超過單檔上限 ${formatMB(MAX_UPLOAD_TOTAL_BYTES)}，請改選較小的圖片`;
}

/** 依序把檔案切批，每批加總 ≤ byte 上限、張數 ≤ 張數上限；呼叫前先用 validateFileSizes 排除單檔超限。 */
export function splitIntoBatches(files: File[]): File[][] {
    const batches: File[][] = [];
    let current: File[] = [];
    let currentSize = 0;
    for (const f of files) {
        if (current.length && (currentSize + f.size > MAX_UPLOAD_TOTAL_BYTES || current.length >= MAX_UPLOAD_FILES_PER_BATCH)) {
            batches.push(current);
            current = [];
            currentSize = 0;
        }
        current.push(f);
        currentSize += f.size;
    }
    if (current.length) batches.push(current);
    return batches;
}

/** 上傳失敗時依錯誤型態給使用者看得懂的訊息。 */
export function uploadErrorMessage(err: unknown): string {
    if ((err as { mark?: string }).mark === TIMEOUT_MARK) return '上傳逾時（網路可能中斷），請再試一次';
    const status = (err as { status?: number }).status;
    if (status === 413) return '圖片總大小超過伺服器限制，請分批上傳';
    return '圖片上傳失敗，請再試一次';
}

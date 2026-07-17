// 單檔大小上限,對齊 server 端最緊的一道：backend RequestBodyLimitLayer 10*1000*1000 bytes
// （nginx client_max_body_size 10M 與 Next server action bodySizeLimit 10mb 都是 10*1024*1024，較寬）。
// 多圖走「一張一請求」逐張上傳(part 數最少避開 WAF 誤殺、後端單張處理遠低於 30 秒逾時),
// 所以請求上限=單檔上限;預留 multipart 邊界/標頭開銷,取 9.5MB。
export const MAX_UPLOAD_FILE_BYTES = 9_500_000;

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

/** 找出單檔就超過上限的圖片（壓縮後仍超限＝大 GIF / 解不開的檔），回錯誤訊息；沒有回 null。 */
export function validateFileSizes(files: File[]): string | null {
    const over = files.filter(f => f.size > MAX_UPLOAD_FILE_BYTES);
    if (!over.length) return null;
    const names = over.slice(0, 3).map(f => `${f.name}（${formatMB(f.size)}）`).join('、');
    const suffix = over.length > 3 ? ` 等 ${over.length} 張` : '';
    return `${names}${suffix} 超過單檔上限 ${formatMB(MAX_UPLOAD_FILE_BYTES)}，請改選較小的圖片`;
}

/** 上傳失敗時依錯誤型態給使用者看得懂的訊息。 */
export function uploadErrorMessage(err: unknown): string {
    const userMessage = (err as { userMessage?: string }).userMessage;
    if (userMessage) return userMessage;
    if ((err as { mark?: string }).mark === TIMEOUT_MARK) return '上傳逾時（網路可能中斷），請再試一次';
    const status = (err as { status?: number }).status;
    if (status === 413) return '圖片總大小超過伺服器限制，請分批上傳';
    return '圖片上傳失敗，請再試一次';
}

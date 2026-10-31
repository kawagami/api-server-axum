// 檔案直傳的 wire 型別與純函式。WS signaling 協定見 monorepo 根 `protocol/transfer-wire.md`；
// DataChannel 上的訊息格式只有前端兩端在用，定義在這裡（`ChannelControl`）。

// 只用 STUN、不設 TURN：打不通就直接告知失敗，檔案永遠不經伺服器
export const ICE_SERVERS: RTCIceServer[] = [
    { urls: ["stun:stun.l.google.com:19302", "stun:stun.cloudflare.com:3478"] },
];

// 不做 trickle ICE：等候選收集完才把整份 SDP 送出，最多等這麼久（只用 STUN 通常 1 秒內）
export const ICE_GATHER_TIMEOUT_MS = 3000;
// 雙方交換完 SDP 後多久還沒連上就判定「無法直連」
export const CONNECT_TIMEOUT_MS = 15000;

// 每則 DataChannel 訊息的大小。16KB 是各瀏覽器互通最保險的值，但 Chrome / Safari / Firefox
// 近年都協商得到 ≥64KB；實際送出時再與 `pc.sctp.maxMessageSize` 取小
export const CHUNK_BYTES = 64 * 1024;
// 從檔案一次讀進記憶體的區塊，再切成 CHUNK_BYTES 送出（逐 64KB 讀檔的 await 開銷太大）
export const READ_BLOCK_BYTES = 1024 * 1024;
// 背壓：送出佇列超過 HIGH 就停，等降到 LOW（bufferedamountlow 事件）再繼續
export const BUFFER_HIGH_BYTES = 8 * 1024 * 1024;
export const BUFFER_LOW_BYTES = 1024 * 1024;

// 接收端整份檔案暫存在記憶體，超過這個總量就在接受前警告（手機分頁可能被系統收掉）
export const RECEIVE_WARN_BYTES = 1024 * 1024 * 1024;

export interface FileMeta {
    name: string;
    size: number;
    type: string;
}

export interface TransferPeer {
    addr: string;
    user_agent: string;
    connected_at: string;
}

// 下行 transfer_offer / transfer_answer / transfer_reject 的 data（後端已把 `to` 換成 `from`）
export interface OfferData {
    from: string;
    id: string;
    sdp: string;
    files: FileMeta[];
}
export interface AnswerData {
    from: string;
    id: string;
    sdp: string;
}
export interface RejectData {
    from: string;
    id: string;
    reason?: string;
}

// DataChannel 上的文字訊息；檔案內容本身是 ArrayBuffer。
// file：接下來的 binary 屬於 offer 裡第 index 個檔案；done：全部送完；ack：接收端確認收齊
export type ChannelControl = { kind: "file"; index: number } | { kind: "done" } | { kind: "ack" };

export type PathKind = "lan" | "stun" | "relay";

export const PATH_LABEL: Record<PathKind, string> = {
    lan: "區網直連",
    stun: "跨網路直連（STUN）",
    relay: "中繼",
};

// 後端 transfer_error 的 reason → 後台文案（查不到顯示原字串）
export const ERROR_LABEL: Record<string, string> = {
    feature_disabled: "此站未開啟檔案傳輸功能",
    not_admin: "目前不是以管理員身分連線，請重新整理頁面",
    not_joined: "尚未加入傳輸，請重新整理頁面",
    peer_not_found: "找不到對方裝置，對方可能已離開此頁",
    too_large: "訊息過大（檔案數量太多）",
};

/** UA → 「iPhone · Safari」這種給人看的裝置名稱（iPadOS 的 Safari 會自稱 Mac，無解） */
export function describeDevice(ua: string): string {
    const os = /iPhone/.test(ua)
        ? "iPhone"
        : /iPad/.test(ua)
          ? "iPad"
          : /Android/.test(ua)
            ? "Android"
            : /Windows/.test(ua)
              ? "Windows"
              : /Macintosh|Mac OS X/.test(ua)
                ? "Mac"
                : /Linux/.test(ua)
                  ? "Linux"
                  : "未知裝置";
    const browser = /Edg(A|iOS)?\//.test(ua)
        ? "Edge"
        : /(CriOS|Chrome)\//.test(ua)
          ? "Chrome"
          : /(FxiOS|Firefox)\//.test(ua)
            ? "Firefox"
            : /Safari\//.test(ua)
              ? "Safari"
              : "瀏覽器";
    return `${os} · ${browser}`;
}

export function toMeta(files: File[]): FileMeta[] {
    return files.map((f) => ({ name: f.name, size: f.size, type: f.type }));
}

export function totalSize(files: { size: number }[]): number {
    return files.reduce((sum, f) => sum + f.size, 0);
}

export function parseControl(raw: string): ChannelControl | null {
    try {
        const msg = JSON.parse(raw) as ChannelControl;
        return msg && typeof msg === "object" && "kind" in msg ? msg : null;
    } catch {
        return null;
    }
}

/** 從 getStats 找出實際選用的候選對，判斷走哪條路 */
export async function detectPath(pc: RTCPeerConnection): Promise<PathKind | null> {
    const stats = await pc.getStats();
    let pairId: string | undefined;
    stats.forEach((r) => {
        if (r.type === "transport" && r.selectedCandidatePairId) pairId = r.selectedCandidatePairId;
    });
    // Firefox 沒有 transport.selectedCandidatePairId，改找 nominated 且成功的那一對
    if (!pairId) {
        stats.forEach((r) => {
            if (r.type === "candidate-pair" && r.nominated && r.state === "succeeded") pairId = r.id;
        });
    }
    const pair = pairId ? stats.get(pairId) : undefined;
    const local = pair ? stats.get(pair.localCandidateId) : undefined;
    switch (local?.candidateType) {
        case "host":
            return "lan";
        case "srflx":
        case "prflx":
            return "stun";
        case "relay":
            return "relay";
        default:
            return null;
    }
}

/** 等 ICE 候選收集完（或逾時），之後 localDescription 就帶齊全部候選 */
export function waitIceGathering(pc: RTCPeerConnection, timeoutMs = ICE_GATHER_TIMEOUT_MS): Promise<void> {
    if (pc.iceGatheringState === "complete") return Promise.resolve();
    return new Promise((resolve) => {
        const done = () => {
            clearTimeout(timer);
            pc.removeEventListener("icegatheringstatechange", onChange);
            resolve();
        };
        const onChange = () => {
            if (pc.iceGatheringState === "complete") done();
        };
        const timer = setTimeout(done, timeoutMs);
        pc.addEventListener("icegatheringstatechange", onChange);
    });
}

/** 送出佇列降到門檻以下（或 channel 關閉）時 resolve */
export function waitBufferLow(dc: RTCDataChannel): Promise<void> {
    return new Promise((resolve) => {
        const done = () => {
            dc.removeEventListener("bufferedamountlow", done);
            dc.removeEventListener("close", done);
            resolve();
        };
        dc.addEventListener("bufferedamountlow", done);
        dc.addEventListener("close", done);
    });
}

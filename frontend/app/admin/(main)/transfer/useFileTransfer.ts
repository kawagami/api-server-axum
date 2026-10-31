"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useWsContext } from "@/libs/ws-context";
import { useWsSubscribe } from "@/hooks/useWsSubscribe";
import {
    BUFFER_HIGH_BYTES,
    BUFFER_LOW_BYTES,
    CHUNK_BYTES,
    CONNECT_TIMEOUT_MS,
    ERROR_LABEL,
    ICE_SERVERS,
    READ_BLOCK_BYTES,
    detectPath,
    parseControl,
    toMeta,
    totalSize,
    waitBufferLow,
    waitIceGathering,
    type AnswerData,
    type FileMeta,
    type OfferData,
    type PathKind,
    type RejectData,
    type TransferPeer,
} from "./transfer-model";

// offering：送出 offer 等對方接受｜incoming：收到 offer 等自己決定｜connecting：交換完 SDP 打洞中
export type Phase = "offering" | "incoming" | "connecting" | "transferring" | "done" | "failed";

export interface TransferSession {
    id: string;
    role: "send" | "receive";
    peer: string;
    files: FileMeta[];
    total: number;
    phase: Phase;
    bytes: number;
    startedAt: number | null;
    // 平均速度 bytes/s（自 startedAt 起算），在進度事件裡算好，render 不碰 Date.now()
    speed: number;
    path: PathKind | null;
    error: string | null;
    received: File[];
}

const NO_DIRECT = "無法直連（雙方網路打不通），請改用同一個 Wi-Fi 再試";
const LOST = "連線中斷，傳輸未完成";
const PROGRESS_PAINT_MS = 200;

// 一次只處理一個傳輸（送或收）；處理中收到的新 offer 一律回 busy 拒絕。
// 已建立的 DataChannel 與 WS 無關 —— WS 重連不影響進行中的傳輸，只需重新 transfer_join。
export function useFileTransfer() {
    const ws = useWsContext();
    const [peers, setPeers] = useState<TransferPeer[]>([]);
    const [session, setSession] = useState<TransferSession | null>(null);
    const [error, setError] = useState<string | null>(null);

    // 事件 handler 都讀 ref：WebRTC 回呼的閉包活得比任何一次 render 都久
    const sessionRef = useRef<TransferSession | null>(null);
    const pcRef = useRef<RTCPeerConnection | null>(null);
    const dcRef = useRef<RTCDataChannel | null>(null);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const wakeLockRef = useRef<WakeLockSentinel | null>(null);
    const filesRef = useRef<File[]>([]);
    const offerSdpRef = useRef<string | null>(null);
    const recvRef = useRef<{ index: number; chunks: ArrayBuffer[]; got: number } | null>(null);
    const progressRef = useRef({ bytes: 0, paintedAt: 0 });

    const commit = useCallback((next: TransferSession | null) => {
        sessionRef.current = next;
        setSession(next);
    }, []);

    const patch = useCallback(
        (p: Partial<TransferSession>) => {
            if (sessionRef.current) commit({ ...sessionRef.current, ...p });
        },
        [commit],
    );

    const teardown = useCallback(() => {
        if (timerRef.current) clearTimeout(timerRef.current);
        timerRef.current = null;
        const dc = dcRef.current;
        if (dc) dc.onopen = dc.onmessage = dc.onclose = null;
        dc?.close();
        const pc = pcRef.current;
        if (pc) pc.onconnectionstatechange = pc.ondatachannel = null;
        pc?.close();
        dcRef.current = pcRef.current = null;
        void wakeLockRef.current?.release().catch(() => {});
        wakeLockRef.current = null;
        recvRef.current = null;
        offerSdpRef.current = null;
    }, []);

    const fail = useCallback(
        (message: string) => {
            teardown();
            patch({ phase: "failed", error: message });
        },
        [teardown, patch],
    );

    const addProgress = (n: number) => {
        const p = progressRef.current;
        p.bytes += n;
        const now = Date.now();
        const s = sessionRef.current;
        if (s && (now - p.paintedAt >= PROGRESS_PAINT_MS || p.bytes >= s.total)) {
            p.paintedAt = now;
            const secs = s.startedAt ? (now - s.startedAt) / 1000 : 0;
            patch({ bytes: p.bytes, speed: secs > 0 ? p.bytes / secs : 0 });
        }
    };

    const startTransferring = (pc: RTCPeerConnection) => {
        if (timerRef.current) clearTimeout(timerRef.current);
        timerRef.current = null;
        progressRef.current = { bytes: 0, paintedAt: 0 };
        patch({ phase: "transferring", startedAt: Date.now(), bytes: 0 });
        void detectPath(pc).then((path) => patch({ path })).catch(() => {});
        // 手機鎖螢幕 / 切 App 會斷線；iOS 16.4+ 支援，不支援就算了
        void navigator.wakeLock?.request("screen").then((lock) => {
            wakeLockRef.current = lock;
        }).catch(() => {});
    };

    const createPeer = () => {
        const pc = new RTCPeerConnection({ iceServers: ICE_SERVERS });
        pc.onconnectionstatechange = () => {
            const phase = sessionRef.current?.phase;
            // 完成後對方關閉連線也會走到這裡，不能把 done 蓋成 failed
            if (pc.connectionState === "failed" && phase !== "done") fail(phase === "transferring" ? LOST : NO_DIRECT);
        };
        pcRef.current = pc;
        return pc;
    };

    const startConnectTimer = () => {
        timerRef.current = setTimeout(() => {
            if (sessionRef.current?.phase === "connecting") fail(NO_DIRECT);
        }, CONNECT_TIMEOUT_MS);
    };

    // ---- 送出端 ----

    const pumpFiles = async (pc: RTCPeerConnection, dc: RTCDataChannel) => {
        startTransferring(pc);
        const chunk = Math.min(CHUNK_BYTES, pc.sctp?.maxMessageSize || CHUNK_BYTES);
        try {
            for (const [index, file] of filesRef.current.entries()) {
                dc.send(JSON.stringify({ kind: "file", index }));
                for (let off = 0; off < file.size; off += READ_BLOCK_BYTES) {
                    const block = await file.slice(off, off + READ_BLOCK_BYTES).arrayBuffer();
                    for (let p = 0; p < block.byteLength; p += chunk) {
                        if (dc.bufferedAmount > BUFFER_HIGH_BYTES) await waitBufferLow(dc);
                        if (dc.readyState !== "open") return;
                        const piece = block.slice(p, p + chunk);
                        dc.send(piece);
                        addProgress(piece.byteLength);
                    }
                }
            }
            dc.send(JSON.stringify({ kind: "done" }));
        } catch {
            // channel 中途關閉時 send 會丟例外；失敗由 onclose 統一回報
        }
    };

    const send = async (peer: string, files: File[]) => {
        if (files.length === 0 || isBusy(sessionRef.current)) return;
        setError(null);
        const id = crypto.randomUUID();
        const metas = toMeta(files);
        filesRef.current = files;
        commit(newSession(id, "send", peer, metas, "offering"));
        try {
            const pc = createPeer();
            const dc = pc.createDataChannel("files", { ordered: true });
            dc.binaryType = "arraybuffer";
            dc.bufferedAmountLowThreshold = BUFFER_LOW_BYTES;
            dc.onopen = () => void pumpFiles(pc, dc);
            dc.onmessage = (e) => {
                if (typeof e.data === "string" && parseControl(e.data)?.kind === "ack") {
                    patch({ phase: "done", bytes: sessionRef.current?.total ?? 0 });
                    teardown();
                }
            };
            dc.onclose = () => {
                if (sessionRef.current?.phase !== "done") fail(LOST);
            };
            dcRef.current = dc;
            await pc.setLocalDescription(await pc.createOffer());
            await waitIceGathering(pc);
            if (sessionRef.current?.id !== id || !pc.localDescription) return;
            ws.send("transfer_offer", { to: peer, id, sdp: pc.localDescription.sdp, files: metas });
        } catch {
            fail("建立連線失敗");
        }
    };

    // ---- 接收端 ----

    const setupReceiver = (pc: RTCPeerConnection, dc: RTCDataChannel) => {
        dcRef.current = dc;
        dc.binaryType = "arraybuffer";
        dc.onopen = () => startTransferring(pc);
        dc.onclose = () => {
            if (sessionRef.current?.phase === "done") teardown();
            else fail(LOST);
        };
        dc.onmessage = (e) => {
            const s = sessionRef.current;
            if (!s) return;
            if (typeof e.data === "string") {
                const ctrl = parseControl(e.data);
                if (ctrl?.kind === "file") {
                    recvRef.current = { index: ctrl.index, chunks: [], got: 0 };
                    if (s.files[ctrl.index]?.size === 0) finishFile();
                } else if (ctrl?.kind === "done") {
                    dc.send(JSON.stringify({ kind: "ack" }));
                    // 不在這裡關：等送出端收到 ack 後關閉，這邊的 onclose 再收尾，ack 才不會被吃掉
                    patch({ phase: "done", bytes: s.total });
                }
                return;
            }
            const cur = recvRef.current;
            if (!cur) return;
            const buf = e.data as ArrayBuffer;
            cur.chunks.push(buf);
            cur.got += buf.byteLength;
            addProgress(buf.byteLength);
            if (cur.got >= (s.files[cur.index]?.size ?? 0)) finishFile();
        };
    };

    const finishFile = () => {
        const s = sessionRef.current;
        const cur = recvRef.current;
        const meta = cur ? s?.files[cur.index] : undefined;
        if (!s || !cur || !meta) return;
        const file = new File(cur.chunks, meta.name, { type: meta.type || "application/octet-stream" });
        recvRef.current = null;
        patch({ received: [...s.received, file] });
    };

    const accept = async () => {
        const s = sessionRef.current;
        const sdp = offerSdpRef.current;
        if (!s || s.phase !== "incoming" || !sdp) return;
        patch({ phase: "connecting" });
        try {
            const pc = createPeer();
            pc.ondatachannel = (e) => setupReceiver(pc, e.channel);
            await pc.setRemoteDescription({ type: "offer", sdp });
            await pc.setLocalDescription(await pc.createAnswer());
            await waitIceGathering(pc);
            if (sessionRef.current?.id !== s.id || !pc.localDescription) return;
            ws.send("transfer_answer", { to: s.peer, id: s.id, sdp: pc.localDescription.sdp });
            startConnectTimer();
        } catch {
            fail("建立連線失敗");
        }
    };

    // 拒絕 / 取消 / 關閉結果 共用：還沒連上的話通知對方，然後收掉
    const dismiss = () => {
        const s = sessionRef.current;
        if (s && (s.phase === "offering" || s.phase === "incoming" || s.phase === "connecting")) {
            ws.send("transfer_reject", { to: s.peer, id: s.id });
        }
        teardown();
        commit(null);
    };

    // ---- WS signaling ----

    useWsSubscribe("transfer_peers", (data) => {
        const list = (data as { peers: TransferPeer[] }).peers ?? [];
        setPeers(list);
        const s = sessionRef.current;
        if (s && (s.phase === "offering" || s.phase === "incoming") && !list.some((p) => p.addr === s.peer)) {
            fail("對方已離開傳輸頁");
        }
    });

    useWsSubscribe("transfer_offer", (data) => {
        const offer = data as OfferData;
        if (isBusy(sessionRef.current)) {
            ws.send("transfer_reject", { to: offer.from, id: offer.id, reason: "busy" });
            return;
        }
        teardown();
        offerSdpRef.current = offer.sdp;
        commit(newSession(offer.id, "receive", offer.from, offer.files, "incoming"));
    });

    useWsSubscribe("transfer_answer", (data) => {
        const answer = data as AnswerData;
        const s = sessionRef.current;
        const pc = pcRef.current;
        if (!s || !pc || s.id !== answer.id || s.phase !== "offering") return;
        patch({ phase: "connecting" });
        startConnectTimer();
        pc.setRemoteDescription({ type: "answer", sdp: answer.sdp }).catch(() => fail("建立連線失敗"));
    });

    useWsSubscribe("transfer_reject", (data) => {
        const reject = data as RejectData;
        const s = sessionRef.current;
        if (!s || s.id !== reject.id || !isBusy(s)) return;
        fail(reject.reason === "busy" ? "對方正在處理另一個傳輸" : "對方已拒絕或取消");
    });

    useWsSubscribe("transfer_error", (data) => {
        const reason = (data as { reason: string }).reason;
        const message = ERROR_LABEL[reason] ?? reason;
        if (isBusy(sessionRef.current)) fail(message);
        else setError(message);
    });

    // 進頁加入、離頁退出；WS 重連後 server 端的房間狀態已遺失，要重新加入
    useEffect(() => {
        ws.send("transfer_join");
        const off = ws.onReconnect(() => ws.send("transfer_join"));
        return () => {
            off();
            ws.send("transfer_leave");
            teardown();
        };
    }, [ws, teardown]);

    return { peers, session, error, send, accept, dismiss };
}

function newSession(id: string, role: "send" | "receive", peer: string, files: FileMeta[], phase: Phase): TransferSession {
    return {
        id,
        role,
        peer,
        files,
        total: totalSize(files),
        phase,
        bytes: 0,
        startedAt: null,
        speed: 0,
        path: null,
        error: null,
        received: [],
    };
}

function isBusy(s: TransferSession | null): boolean {
    return !!s && s.phase !== "done" && s.phase !== "failed";
}

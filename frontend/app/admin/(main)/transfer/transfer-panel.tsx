"use client";

import { useRef, useState, useSyncExternalStore } from "react";
import { Download, FileUp, Loader2, Send, Share2, Smartphone, X } from "lucide-react";
import ErrorBanner from "@/components/admin/error-banner";
import { formatBytes } from "@/libs/format-bytes";
import { formatTimeOfDay } from "@/libs/admin-datetime";
import { cn } from "@/libs/cn";
import { useFileTransfer, type TransferSession } from "./useFileTransfer";
import { PATH_LABEL, RECEIVE_WARN_BYTES, describeDevice, totalSize, type TransferPeer } from "./transfer-model";

const CARD = "bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 flex flex-col gap-3";
const BTN = "inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm transition-colors disabled:opacity-50 disabled:cursor-not-allowed";
const BTN_PRIMARY = cn(BTN, "bg-primary-600 text-white hover:bg-primary-700");
const BTN_GHOST = cn(BTN, "border border-neutral-300 dark:border-neutral-600 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800");

// RTCPeerConnection 只在瀏覽器存在；server render 時當作支援，避免 hydration 不一致
const subscribeNothing = () => () => {};
function useWebRtcSupported() {
    return useSyncExternalStore(
        subscribeNothing,
        () => typeof RTCPeerConnection !== "undefined",
        () => true,
    );
}

export default function TransferPanel() {
    const { peers, session, error, send, accept, dismiss } = useFileTransfer();
    const [files, setFiles] = useState<File[]>([]);
    const inputRef = useRef<HTMLInputElement>(null);
    const supported = useWebRtcSupported();
    const busy = !!session && session.phase !== "done" && session.phase !== "failed";

    const peerName = (addr: string) => {
        const p = peers.find((x) => x.addr === addr);
        return p ? describeDevice(p.user_agent) : "對方裝置";
    };

    return (
        <div className="flex flex-col gap-4 max-w-2xl">
            <ErrorBanner message={supported ? error : "此瀏覽器不支援 WebRTC，無法使用檔案直傳"} />

            {session && (
                <SessionCard session={session} peerName={peerName(session.peer)} onAccept={accept} onDismiss={dismiss} />
            )}

            <section className={CARD}>
                <h2 className="font-medium text-neutral-800 dark:text-neutral-100">1. 選擇檔案</h2>
                <input
                    ref={inputRef}
                    type="file"
                    multiple
                    className="hidden"
                    onChange={(e) => setFiles(Array.from(e.target.files ?? []))}
                />
                <div className="flex flex-wrap items-center gap-2">
                    <button type="button" className={BTN_GHOST} onClick={() => inputRef.current?.click()}>
                        <FileUp className="size-4" />
                        選擇圖片 / 影片
                    </button>
                    {files.length > 0 && (
                        <>
                            <span className="text-sm text-neutral-600 dark:text-neutral-300">
                                {files.length} 個檔案，共 {formatBytes(totalSize(files))}
                            </span>
                            <button
                                type="button"
                                className={BTN_GHOST}
                                onClick={() => {
                                    setFiles([]);
                                    if (inputRef.current) inputRef.current.value = "";
                                }}
                            >
                                清除
                            </button>
                        </>
                    )}
                </div>
            </section>

            <section className={CARD}>
                <h2 className="font-medium text-neutral-800 dark:text-neutral-100">2. 傳給哪台裝置</h2>
                {peers.length === 0 ? (
                    <p className="text-sm text-neutral-500 dark:text-neutral-400">
                        目前沒有其他裝置。在另一台裝置用同一個帳號登入後台，打開這一頁就會出現在這裡。
                    </p>
                ) : (
                    <ul className="flex flex-col divide-y divide-neutral-200 dark:divide-neutral-700">
                        {peers.map((p) => (
                            <PeerRow
                                key={p.addr}
                                peer={p}
                                disabled={!supported || busy || files.length === 0}
                                onSend={() => void send(p.addr, files)}
                            />
                        ))}
                    </ul>
                )}
            </section>

            <p className="text-xs text-neutral-500 dark:text-neutral-400">
                檔案在兩台裝置之間直接傳送，不經過伺服器。雙方在同一個 Wi-Fi 最穩；跨網路（例如手機用行動網路）可能打不通。
                傳輸中請讓兩邊都停留在這一頁，手機不要鎖螢幕或切到其他 App。
            </p>
        </div>
    );
}

function PeerRow({ peer, disabled, onSend }: { peer: TransferPeer; disabled: boolean; onSend: () => void }) {
    return (
        <li className="flex items-center justify-between gap-3 py-2">
            <div className="flex min-w-0 items-center gap-2">
                <Smartphone className="size-4 shrink-0 text-neutral-500" />
                <div className="min-w-0">
                    <p className="text-sm text-neutral-800 dark:text-neutral-100">{describeDevice(peer.user_agent)}</p>
                    <p className="text-xs text-neutral-500 dark:text-neutral-400">{formatTimeOfDay(peer.connected_at)} 連線</p>
                </div>
            </div>
            <button type="button" className={BTN_PRIMARY} disabled={disabled} onClick={onSend}>
                <Send className="size-4" />
                傳送
            </button>
        </li>
    );
}

function SessionCard({
    session: s,
    peerName,
    onAccept,
    onDismiss,
}: {
    session: TransferSession;
    peerName: string;
    onAccept: () => void;
    onDismiss: () => void;
}) {
    const title =
        s.role === "send" ? `傳送給 ${peerName}` : `來自 ${peerName}`;

    return (
        <section className={cn(CARD, "border-primary-300 dark:border-primary-700")} aria-live="polite">
            <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                    <h2 className="font-medium text-neutral-800 dark:text-neutral-100">{title}</h2>
                    <p className="text-sm text-neutral-500 dark:text-neutral-400">
                        {s.files.length} 個檔案，共 {formatBytes(s.total)}
                        {s.path && ` · ${PATH_LABEL[s.path]}`}
                    </p>
                </div>
                {s.phase !== "incoming" && (
                    <button type="button" className={BTN_GHOST} onClick={onDismiss}>
                        <X className="size-4" />
                        {s.phase === "done" || s.phase === "failed" ? "關閉" : "取消"}
                    </button>
                )}
            </div>

            {s.phase === "incoming" && (
                <>
                    <FileList names={s.files.map((f) => `${f.name}（${formatBytes(f.size)}）`)} />
                    {s.total > RECEIVE_WARN_BYTES && (
                        <p className="text-sm text-amber-700 dark:text-amber-400">
                            檔案較大：接收時會整份暫存在記憶體，手機上可能因記憶體不足而中斷。
                        </p>
                    )}
                    <div className="flex gap-2">
                        <button type="button" className={BTN_PRIMARY} onClick={onAccept}>接受</button>
                        <button type="button" className={BTN_GHOST} onClick={onDismiss}>拒絕</button>
                    </div>
                </>
            )}

            {(s.phase === "offering" || s.phase === "connecting") && (
                <p className="flex items-center gap-2 text-sm text-neutral-600 dark:text-neutral-300">
                    <Loader2 className="size-4 animate-spin" />
                    {s.phase === "offering" ? "等待對方接受…" : "建立直連中…"}
                </p>
            )}

            {s.phase === "transferring" && <Progress session={s} />}

            {s.phase === "failed" && <p className="text-sm text-red-600 dark:text-red-400">{s.error}</p>}

            {s.phase === "done" && s.role === "send" && (
                <p className="text-sm text-neutral-700 dark:text-neutral-200">傳送完成</p>
            )}
            {s.received.length > 0 && <ReceivedFiles files={s.received} />}
        </section>
    );
}

function Progress({ session: s }: { session: TransferSession }) {
    const pct = s.total > 0 ? Math.min(100, (s.bytes / s.total) * 100) : 100;
    return (
        <div className="flex flex-col gap-1">
            <div
                className="h-2 w-full overflow-hidden rounded-full bg-neutral-200 dark:bg-neutral-700"
                role="progressbar"
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={Math.round(pct)}
            >
                <div className="h-full bg-primary-500 transition-[width]" style={{ width: `${pct}%` }} />
            </div>
            <p className="text-xs text-neutral-500 dark:text-neutral-400">
                {formatBytes(s.bytes)} / {formatBytes(s.total)}（{pct.toFixed(0)}%）
                {s.speed > 0 && ` · ${formatBytes(s.speed)}/s`}
            </p>
        </div>
    );
}

function FileList({ names }: { names: string[] }) {
    return (
        <ul className="max-h-40 overflow-auto text-sm text-neutral-700 dark:text-neutral-200">
            {names.map((n, i) => (
                <li key={i} className="truncate">{n}</li>
            ))}
        </ul>
    );
}

function ReceivedFiles({ files }: { files: File[] }) {
    const canShare = typeof navigator !== "undefined" && !!navigator.canShare?.({ files });

    const download = (file: File) => {
        const url = URL.createObjectURL(file);
        const a = document.createElement("a");
        a.href = url;
        a.download = file.name;
        a.click();
        setTimeout(() => URL.revokeObjectURL(url), 60_000);
    };

    // 手機上 share sheet 才有「儲存影像 / 儲存到相簿」；使用者取消分享會 reject，忽略即可
    const share = (list: File[]) => void navigator.share({ files: list }).catch(() => {});

    return (
        <div className="flex flex-col gap-2">
            {canShare && files.length > 1 && (
                <button type="button" className={cn(BTN_PRIMARY, "self-start")} onClick={() => share(files)}>
                    <Share2 className="size-4" />
                    全部分享 / 儲存到相簿
                </button>
            )}
            <ul className="flex flex-col divide-y divide-neutral-200 dark:divide-neutral-700">
                {files.map((f, i) => (
                    <li key={i} className="flex items-center justify-between gap-3 py-2">
                        <span className="min-w-0 truncate text-sm text-neutral-800 dark:text-neutral-100">
                            {f.name}（{formatBytes(f.size)}）
                        </span>
                        <span className="flex shrink-0 gap-2">
                            {canShare && (
                                <button type="button" className={BTN_GHOST} onClick={() => share([f])} aria-label={`分享 ${f.name}`}>
                                    <Share2 className="size-4" />
                                </button>
                            )}
                            <button type="button" className={BTN_GHOST} onClick={() => download(f)} aria-label={`下載 ${f.name}`}>
                                <Download className="size-4" />
                            </button>
                        </span>
                    </li>
                ))}
            </ul>
        </div>
    );
}

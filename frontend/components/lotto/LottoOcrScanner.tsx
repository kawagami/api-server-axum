"use client";

import { useEffect, useRef, useState } from "react";
import { useTranslations } from "next-intl";
import { Loader2, ScanText, ImageIcon, ScanLine } from "lucide-react";

interface Props {
    // 回傳 true = 此次辨識已被接受並停止；false = 沒讀到號碼，繼續掃（同 QR scanner 的 onDecoded）
    onDecoded: (text: string) => boolean;
}

type Sub = 'live' | 'photo';

// 共用：數字白名單提升票面號碼辨識率
const WHITELIST = '0123456789/()$. :';

// 即時相機 OCR（節流，~每 0.7s 一幀）+ 拍照備援；tesseract.js 動態載入、SSR 安全
export default function LottoOcrScanner({ onDecoded }: Props) {
    const t = useTranslations('Lotto');
    const [sub, setSub] = useState<Sub>('live');

    // callback / t 收進 ref，避免識別改變導致 effect 重啟相機
    const onDecodedRef = useRef(onDecoded);
    const tRef = useRef(t);
    useEffect(() => { onDecodedRef.current = onDecoded; tRef.current = t; });

    // --- 即時掃描 ---
    const videoRef = useRef<HTMLVideoElement>(null);
    const [starting, setStarting] = useState(true);
    const [liveError, setLiveError] = useState('');

    useEffect(() => {
        if (sub !== 'live') return;
        let cancelled = false;
        let accepted = false;
        let stream: MediaStream | null = null;
        // tesseract worker 重用（每幀重建會很慢）
        let worker: Awaited<ReturnType<typeof import('tesseract.js')['createWorker']>> | null = null;
        let timer: ReturnType<typeof setTimeout> | null = null;
        const canvas = document.createElement('canvas');

        (async () => {
            setLiveError('');
            setStarting(true);
            try {
                const Tesseract = await import('tesseract.js');
                worker = await Tesseract.createWorker('eng', 1);
                await worker.setParameters({ tessedit_char_whitelist: WHITELIST });
                if (cancelled) { await worker.terminate(); return; }

                stream = await navigator.mediaDevices.getUserMedia({
                    video: { facingMode: 'environment', width: { ideal: 1920 }, height: { ideal: 1080 } },
                });
                const video = videoRef.current;
                if (!video || cancelled) { stream.getTracks().forEach(tr => tr.stop()); await worker.terminate(); return; }
                video.srcObject = stream;
                await video.play();
                try {
                    const track = stream.getVideoTracks()[0];
                    await track.applyConstraints({ advanced: [{ focusMode: 'continuous' }] } as unknown as MediaTrackConstraints);
                } catch { /* 對焦控制非所有裝置支援 */ }
                setStarting(false);

                const loop = async () => {
                    if (cancelled || accepted) return;
                    try {
                        const v = videoRef.current;
                        if (v && v.videoWidth && worker) {
                            // 裁中央橫帶（號碼/日期區）+ 縮圖加速
                            const vw = v.videoWidth, vh = v.videoHeight;
                            const cropY = Math.round(vh * 0.15), cropH = Math.round(vh * 0.7);
                            const scale = Math.min(1, 1080 / vw);
                            canvas.width = Math.round(vw * scale);
                            canvas.height = Math.round(cropH * scale);
                            const ctx = canvas.getContext('2d');
                            ctx?.drawImage(v, 0, cropY, vw, cropH, 0, 0, canvas.width, canvas.height);
                            const { data } = await worker.recognize(canvas);
                            if (!cancelled && !accepted && onDecodedRef.current(data.text)) {
                                accepted = true; // 父層會切回 manual → 本元件 unmount → cleanup 收掉相機/worker
                                return;
                            }
                        }
                    } catch { /* 單幀失敗忽略，續掃 */ }
                    if (!cancelled && !accepted) timer = setTimeout(loop, 700);
                };
                loop();
            } catch {
                if (!cancelled) { setLiveError(tRef.current('ocrCameraError')); setStarting(false); }
            }
        })();

        return () => {
            cancelled = true;
            if (timer) clearTimeout(timer);
            stream?.getTracks().forEach(tr => tr.stop());
            worker?.terminate();
        };
    }, [sub]);

    // --- 拍照備援 ---
    const inputRef = useRef<HTMLInputElement>(null);
    const [preview, setPreview] = useState<string | null>(null);
    const [running, setRunning] = useState(false);
    const [progress, setProgress] = useState(0);
    const [photoMsg, setPhotoMsg] = useState('');

    async function handleFile(e: React.ChangeEvent<HTMLInputElement>) {
        const file = e.target.files?.[0];
        if (!file) return;
        setPhotoMsg('');
        const url = URL.createObjectURL(file);
        setPreview(prev => { if (prev) URL.revokeObjectURL(prev); return url; });
        setRunning(true);
        setProgress(0);
        try {
            const Tesseract = await import('tesseract.js');
            const worker = await Tesseract.createWorker('eng', 1, {
                logger: m => { if (m.status === 'recognizing text') setProgress(Math.round(m.progress * 100)); },
            });
            await worker.setParameters({ tessedit_char_whitelist: WHITELIST });
            const { data } = await worker.recognize(file);
            await worker.terminate();
            const ok = onDecodedRef.current(data.text);
            if (!ok) setPhotoMsg(tRef.current('ocrNotFound'));
        } catch {
            setPhotoMsg(tRef.current('ocrError'));
        } finally {
            setRunning(false);
            if (inputRef.current) inputRef.current.value = '';
        }
    }

    return (
        <div className="flex flex-col gap-3">
            {/* live / photo 切換 */}
            <div className="flex gap-2 text-sm">
                <button
                    type="button"
                    onClick={() => setSub('live')}
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full transition-colors ${sub === 'live'
                        ? 'bg-primary-500 text-white'
                        : 'border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}`}
                >
                    <ScanLine size={15} />
                    {t('ocrUseLive')}
                </button>
                <button
                    type="button"
                    onClick={() => setSub('photo')}
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full transition-colors ${sub === 'photo'
                        ? 'bg-primary-500 text-white'
                        : 'border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}`}
                >
                    <ImageIcon size={15} />
                    {t('ocrUsePhoto')}
                </button>
            </div>

            {sub === 'live' ? (
                <>
                    <div className="relative aspect-video w-full max-w-md mx-auto rounded-xl overflow-hidden bg-neutral-900">
                        <video ref={videoRef} className="w-full h-full object-cover" playsInline muted />
                        {/* 取景框：中央橫帶（對應 OCR 裁切區） */}
                        <div className="pointer-events-none absolute inset-x-4 inset-y-[15%] border-2 border-white/70 rounded-lg" />
                        {!liveError && (
                            <div className="absolute bottom-2 inset-x-0 flex justify-center">
                                <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-black/55 text-white text-xs">
                                    <Loader2 className="animate-spin" size={12} />
                                    {starting ? t('ocrStarting') : t('ocrScanning')}
                                </span>
                            </div>
                        )}
                    </div>
                    {liveError
                        ? <p className="text-red-500 text-sm text-center">{liveError}</p>
                        : <p className="text-neutral-500 dark:text-neutral-400 text-sm text-center">{t('ocrLiveHint')}</p>}
                </>
            ) : (
                <>
                    <input ref={inputRef} type="file" accept="image/*" capture="environment" onChange={handleFile} className="hidden" />
                    <button
                        type="button"
                        onClick={() => inputRef.current?.click()}
                        disabled={running}
                        className="flex items-center justify-center gap-2 py-3 rounded-xl border border-dashed border-neutral-300 dark:border-neutral-600 hover:bg-neutral-50 dark:hover:bg-neutral-800 disabled:opacity-50 text-sm"
                    >
                        {running ? <Loader2 className="animate-spin" size={18} /> : <ImageIcon size={18} />}
                        {running ? t('ocrRunning', { progress }) : t('ocrPick')}
                    </button>
                    {preview && (
                        // eslint-disable-next-line @next/next/no-img-element
                        <img src={preview} alt="" className="max-h-48 w-auto mx-auto rounded-lg border dark:border-neutral-700" />
                    )}
                    {photoMsg && <p className="text-red-500 text-sm text-center">{photoMsg}</p>}
                </>
            )}

            <p className="text-xs text-neutral-400 dark:text-neutral-500 flex items-start gap-1.5">
                <ScanText size={14} className="shrink-0 mt-0.5" />
                {t('ocrHint')}
            </p>
        </div>
    );
}

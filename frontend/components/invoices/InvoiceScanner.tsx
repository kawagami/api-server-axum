"use client";

import { useEffect, useRef, useState } from "react";
import { useTranslations } from "next-intl";
import { Loader2 } from "lucide-react";
import type { IScannerControls } from "@zxing/browser";

interface Props {
    mode: 'qr' | 'barcode';
    // 回傳 true = 已接受並停止掃描；false = 此次解碼不符（如發票右段 QR），靜默繼續掃
    onDecoded: (text: string) => boolean;
}

// 相機掃描；@zxing/browser 動態載入（避免進主 bundle、SSR 安全）
// qr → 左方 QR Code；barcode → 一維條碼（Code 39，多數紙本電子發票）
export default function InvoiceScanner({ mode, onDecoded }: Props) {
    const t = useTranslations('Invoices');
    const videoRef = useRef<HTMLVideoElement>(null);
    const [starting, setStarting] = useState(true);
    const [error, setError] = useState('');
    const acceptedRef = useRef(false);
    // 把 callback / t 收進 ref，避免它們的識別改變導致 effect 重啟相機
    const onDecodedRef = useRef(onDecoded);
    const tRef = useRef(t);
    useEffect(() => {
        onDecodedRef.current = onDecoded;
        tRef.current = t;
    });

    useEffect(() => {
        let controls: IScannerControls | null = null;
        let cancelled = false;
        acceptedRef.current = false;

        (async () => {
            try {
                const { BrowserQRCodeReader, BrowserMultiFormatOneDReader } = await import("@zxing/browser");
                const reader = mode === 'qr'
                    ? new BrowserQRCodeReader()
                    : new BrowserMultiFormatOneDReader();
                const video = videoRef.current;
                if (!video || cancelled) return;
                controls = await reader.decodeFromConstraints(
                    // 高解析度：發票 QR 模組密、低解析度解不出，1080p 大幅提升命中率（ideal 不支援會自動降級）
                    { video: { facingMode: 'environment', width: { ideal: 1920 }, height: { ideal: 1080 } } },
                    video,
                    (result) => {
                        if (result && !acceptedRef.current) {
                            const accepted = onDecodedRef.current(result.getText());
                            if (accepted) {
                                acceptedRef.current = true;
                                controls?.stop();
                            }
                            // 不符則不停、不報錯，讓鏡頭移到正確碼自然鎖上
                        }
                    },
                );
                if (cancelled) {
                    controls.stop();
                    return;
                }
                // 連續自動對焦（部分裝置不支援，try/catch 忽略）
                try {
                    const track = (video.srcObject as MediaStream | null)?.getVideoTracks?.()[0];
                    await track?.applyConstraints({ advanced: [{ focusMode: 'continuous' }] } as unknown as MediaTrackConstraints);
                } catch { /* 對焦控制非所有裝置支援 */ }
                setStarting(false);
            } catch {
                if (!cancelled) {
                    setError(tRef.current('cameraError'));
                    setStarting(false);
                }
            }
        })();

        return () => {
            cancelled = true;
            controls?.stop();
        };
    }, [mode]);

    // 條碼用較寬的取景框（橫向），QR 用方形
    const frameClass = mode === 'barcode'
        ? 'inset-x-6 inset-y-1/3'
        : 'inset-8';

    return (
        <div className="flex flex-col gap-3">
            <div className="relative aspect-square w-full max-w-sm mx-auto rounded-xl overflow-hidden bg-neutral-900">
                <video ref={videoRef} className="w-full h-full object-cover" playsInline muted />
                {/* 取景框 */}
                <div className={`pointer-events-none absolute ${frameClass} border-2 border-white/70 rounded-lg`} />
                {starting && (
                    <div className="absolute inset-0 flex items-center justify-center text-white">
                        <Loader2 className="animate-spin" size={28} />
                    </div>
                )}
            </div>
            {error
                ? <p className="text-red-500 text-sm text-center">{error}</p>
                : <p className="text-neutral-500 dark:text-neutral-400 text-sm text-center">{mode === 'qr' ? t('scanQrHint') : t('scanBarcodeHint')}</p>}
        </div>
    );
}

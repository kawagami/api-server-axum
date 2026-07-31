"use client";

import { useEffect, useMemo, useState } from "react";
import { Loader2, UploadCloud, X } from "lucide-react";
import { uploadProgressLabel, type UploadProgress } from "@/libs/upload-limits";

interface Props {
    fileInputRef: React.RefObject<HTMLInputElement | null>;
    selectedFiles: File[];
    isUploading: boolean;
    uploadProgress: UploadProgress | null;
    uploadError: string | null;
    canUpload: boolean;
    onAddFiles: (files: File[]) => void;
    onImageChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onRemoveSelectedAt: (index: number) => void;
    onRemoveSelectedImage: () => void;
    onUpload: () => void;
}

const UploadSection = ({
    fileInputRef, selectedFiles, isUploading, uploadProgress, uploadError, canUpload,
    onAddFiles, onImageChange, onRemoveSelectedAt, onRemoveSelectedImage, onUpload,
}: Props) => {
    const [dragOver, setDragOver] = useState(false);
    const previewUrls = useMemo(() => selectedFiles.map(f => URL.createObjectURL(f)), [selectedFiles]);

    useEffect(() => {
        return () => previewUrls.forEach(u => URL.revokeObjectURL(u));
    }, [previewUrls]);

    const openPicker = () => fileInputRef.current?.click();

    const onDrop = (e: React.DragEvent) => {
        e.preventDefault();
        setDragOver(false);
        if (e.dataTransfer.files?.length) onAddFiles(Array.from(e.dataTransfer.files));
    };

    return (
        <div>
            <input ref={fileInputRef} accept="image/*" type="file" multiple onChange={onImageChange} className="hidden" />

            {/* 拖放 / 點選上傳區 */}
            <div
                role="button"
                tabIndex={0}
                onClick={openPicker}
                onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); openPicker(); } }}
                onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
                onDragLeave={() => setDragOver(false)}
                onDrop={onDrop}
                className={`flex flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed p-8 text-center cursor-pointer transition-colors ${
                    dragOver
                        ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                        : 'border-neutral-300 dark:border-neutral-600 hover:border-primary-400 hover:bg-neutral-50 dark:hover:bg-neutral-800/50'
                }`}
            >
                <UploadCloud className="w-8 h-8 text-primary-500" />
                <p className="text-sm text-neutral-600 dark:text-neutral-300">
                    <span className="font-semibold text-primary-600 dark:text-primary-400">點選選擇</span> 或將圖片拖放到這裡
                </p>
                <p className="text-xs text-neutral-400">支援多張,上傳前會自動壓縮轉檔</p>
            </div>

            {/* 已選取預覽 */}
            {previewUrls.length > 0 && (
                <div className="mt-4">
                    <div className="flex items-center justify-between mb-2">
                        <span className="text-sm text-neutral-600 dark:text-neutral-300">已選取 {selectedFiles.length} 張</span>
                        <button
                            onClick={onRemoveSelectedImage}
                            className="text-xs text-neutral-400 hover:text-red-600 transition-colors"
                        >
                            清空
                        </button>
                    </div>
                    <div className="flex flex-wrap gap-3">
                        {previewUrls.map((url, i) => (
                            <div key={url} className="relative group">
                                {/* eslint-disable-next-line @next/next/no-img-element -- blob: URL 無法經 next/image 最佳化 */}
                                <img src={url} className="w-24 h-24 object-cover rounded-lg ring-1 ring-neutral-200 dark:ring-neutral-700" alt={`已選取 ${i + 1}`} />
                                <button
                                    onClick={() => onRemoveSelectedAt(i)}
                                    aria-label={`移除已選取 ${i + 1}`}
                                    className="absolute -top-2 -right-2 p-1 rounded-full bg-neutral-700 text-white opacity-0 group-hover:opacity-100 hover:bg-red-600 transition-colors"
                                >
                                    <X size={14} />
                                </button>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {uploadError && <p className="text-red-500 text-sm text-center mt-4">{uploadError}</p>}

            <div className="flex justify-center mt-4">
                <button
                    className={`flex items-center gap-2 py-2 px-6 rounded-lg font-medium transition-colors ${
                        isUploading || !canUpload
                            ? 'bg-neutral-300 dark:bg-neutral-700 text-neutral-500 cursor-not-allowed'
                            : 'bg-primary-600 hover:bg-primary-700 text-white'
                    }`}
                    onClick={onUpload}
                    disabled={isUploading || !canUpload}
                >
                    {isUploading && <Loader2 className="w-4 h-4 animate-spin" />}
                    {isUploading ? uploadProgressLabel(uploadProgress) : '上傳圖片'}
                </button>
            </div>
        </div>
    );
};

export default UploadSection;

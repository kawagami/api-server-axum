"use client";

import { useEffect, useMemo } from "react";
import { Loader2 } from "lucide-react";

interface Props {
    fileInputRef: React.RefObject<HTMLInputElement | null>;
    selectedFiles: File[];
    isUploading: boolean;
    uploadProgress: { current: number; total: number } | null;
    uploadError: string | null;
    canUpload: boolean;
    onImageChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onRemoveSelectedImage: () => void;
    onUpload: () => void;
}

const UploadSection = ({ fileInputRef, selectedFiles, isUploading, uploadProgress, uploadError, canUpload, onImageChange, onRemoveSelectedImage, onUpload }: Props) => {
    const previewUrls = useMemo(() => selectedFiles.map(f => URL.createObjectURL(f)), [selectedFiles]);

    useEffect(() => {
        return () => previewUrls.forEach(u => URL.revokeObjectURL(u));
    }, [previewUrls]);

    return (
        <div className="flex flex-col justify-center items-center pt-4">
            <input ref={fileInputRef} accept="image/*" type="file" multiple onChange={onImageChange} className="mt-4" />
            {previewUrls.length > 0 && (
                <div className="mt-4 flex flex-col items-center gap-4">
                    <div className="flex flex-wrap justify-center gap-4">
                        {previewUrls.map((url, i) => (
                            // eslint-disable-next-line @next/next/no-img-element -- blob: URL 無法經 next/image 最佳化
                            <img key={url} src={url} className="max-w-[200px] max-h-48 rounded-lg" alt={`Selected ${i + 1}`} />
                        ))}
                    </div>
                    <button onClick={onRemoveSelectedImage} className="cursor-pointer py-2 px-4 bg-red-500 text-white rounded">
                        Remove Selected ({selectedFiles.length})
                    </button>
                </div>
            )}
            {uploadError && <p className="text-red-500 text-sm text-center mt-4">{uploadError}</p>}
            <button
                className={`flex items-center gap-1 py-2 px-4 rounded mt-4 ${isUploading || !canUpload ? 'bg-neutral-500 text-white cursor-not-allowed' : 'bg-primary-500 text-white'}`}
                onClick={onUpload}
                disabled={isUploading || !canUpload}
            >
                {isUploading && <Loader2 className="w-4 h-4 animate-spin" />}
                {isUploading ? (uploadProgress ? `上傳中 (${uploadProgress.current}/${uploadProgress.total})...` : '上傳中...') : '上傳圖片'}
            </button>
        </div>
    );
};

export default UploadSection;

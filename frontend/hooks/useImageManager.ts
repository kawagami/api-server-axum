import { useState, useRef } from 'react';
import { uploadImages } from '@/api/images';
import { deleteImage } from '@/api/images';
import { validateFileSizes, splitIntoBatches, uploadErrorMessage, type UploadProgress } from '@/libs/upload-limits';
import { compressImages } from '@/libs/client-image';

export interface ManagedImage {
    name: string;
    url: string;
    status?: string;
}

export const useImageManager = (initialImages: ManagedImage[]) => {
    const [images, setImages] = useState<ManagedImage[]>(initialImages);
    const [deletingImage, setDeletingImage] = useState<string | null>(null);
    const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
    const [isUploading, setIsUploading] = useState(false);
    const [uploadProgress, setUploadProgress] = useState<UploadProgress | null>(null);
    const [uploadError, setUploadError] = useState<string | null>(null);
    const [copiedImage, setCopiedImage] = useState<string | null>(null);

    const fileInputRef = useRef<HTMLInputElement>(null);

    const imageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files?.length) {
            setSelectedFiles(Array.from(e.target.files));
            setUploadError(null);
        }
    };

    const removeSelectedImage = () => {
        setSelectedFiles([]);
        setUploadError(null);
        if (fileInputRef.current) fileInputRef.current.value = '';
    };

    const handleUpload = async () => {
        if (!selectedFiles.length || isUploading) return;
        setIsUploading(true);
        setUploadError(null);
        try {
            const compressed = await compressImages(selectedFiles, (current, total) =>
                setUploadProgress({ phase: 'compress', current, total }));
            // 大小檢查在壓縮後做:多數超限原圖壓完就過了,擋不住的只剩大 GIF / 解不開的檔
            const sizeError = validateFileSizes(compressed);
            if (sizeError) {
                setUploadError(sizeError);
                return;
            }
            // 壓縮檔對映回原檔,批次成功時才能從選取移除對應原檔
            const originalOf = new Map<File, File>();
            compressed.forEach((c, i) => originalOf.set(c, selectedFiles[i]));

            const batches = splitIntoBatches(compressed);
            for (let i = 0; i < batches.length; i++) {
                setUploadProgress({ phase: 'upload', current: i + 1, total: batches.length });
                const formData = new FormData();
                batches[i].forEach(f => formData.append('file', f));

                const responses = await uploadImages(formData);
                const newImages = responses.map(r => ({ name: r.id, url: r.url, status: r.status }));
                setImages((prev) => [...prev, ...newImages]);
                // 已成功的批次移出選取，中途失敗時重按上傳只會送剩下的
                const uploaded = new Set(batches[i].map(f => originalOf.get(f)!));
                setSelectedFiles((prev) => prev.filter(f => !uploaded.has(f)));
            }
            removeSelectedImage();
        } catch (err) {
            console.error('Upload error:', err);
            setUploadError(uploadErrorMessage(err));
        } finally {
            setIsUploading(false);
            setUploadProgress(null);
        }
    };

    const handleDelete = async (fileName: string) => {
        setDeletingImage(fileName);
        try {
            await deleteImage(fileName);
            setImages((prev) => prev.filter((img) => img.name !== fileName));
        } catch (err) {
            console.error('Delete error:', err);
        } finally {
            setDeletingImage(null);
        }
    };

    const handleCopy = async (url: string) => {
        try {
            await navigator.clipboard.writeText(url);
            setCopiedImage(url);
            setTimeout(() => setCopiedImage(null), 2000);
        } catch (err) {
            console.error('Copy error:', err);
        }
    };

    return {
        images,
        deletingImage,
        selectedFiles,
        isUploading,
        uploadProgress,
        uploadError,
        canUpload: selectedFiles.length > 0,
        copiedImage,
        fileInputRef,
        imageChange,
        removeSelectedImage,
        handleUpload,
        handleDelete,
        handleCopy,
    };
};

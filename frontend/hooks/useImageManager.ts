import { useState, useRef } from 'react';
import { uploadImages } from '@/api/images';
import { deleteImage } from '@/api/images';
import { validateFileSizes, uploadErrorMessage, withUploadTimeout, type UploadProgress } from '@/libs/upload-limits';
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
            // 一張一請求逐張上傳:part 數最少(避 WAF 誤殺)、後端單張處理遠低於 30 秒逾時
            for (let i = 0; i < compressed.length; i++) {
                setUploadProgress({ phase: 'upload', current: i + 1, total: compressed.length });
                const formData = new FormData();
                formData.append('file', compressed[i]);

                const responses = await withUploadTimeout(uploadImages(formData));
                const newImages = responses.map(r => ({ name: r.id, url: r.url, status: r.status }));
                setImages((prev) => [...prev, ...newImages]);
                // 已成功的移出選取，中途失敗時重按上傳只會送剩下的
                const original = selectedFiles[i];
                setSelectedFiles((prev) => prev.filter(f => f !== original));
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

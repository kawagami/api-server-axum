import { useState, useRef } from 'react';
import { uploadImages } from '@/api/images';
import { deleteImage } from '@/api/images';
import { validateFileSizes, splitIntoBatches, uploadErrorMessage } from '@/libs/upload-limits';

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
    const [uploadProgress, setUploadProgress] = useState<{ current: number; total: number } | null>(null);
    const [uploadError, setUploadError] = useState<string | null>(null);
    const [copiedImage, setCopiedImage] = useState<string | null>(null);

    const fileInputRef = useRef<HTMLInputElement>(null);

    // 總大小超限會自動切批上傳；只有「單檔就超限」才擋住上傳鈕（隨 selectedFiles 派生）
    const sizeError = validateFileSizes(selectedFiles);

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
        if (!selectedFiles.length || sizeError) return;
        setIsUploading(true);
        setUploadError(null);
        const batches = splitIntoBatches(selectedFiles);
        try {
            for (let i = 0; i < batches.length; i++) {
                if (batches.length > 1) setUploadProgress({ current: i + 1, total: batches.length });
                const formData = new FormData();
                batches[i].forEach(f => formData.append('file', f));

                const responses = await uploadImages(formData);
                const newImages = responses.map(r => ({ name: r.id, url: r.url, status: r.status }));
                setImages((prev) => [...prev, ...newImages]);
                // 已成功的批次移出選取，中途失敗時重按上傳只會送剩下的
                const uploaded = new Set(batches[i]);
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
        uploadError: sizeError ?? uploadError,
        canUpload: selectedFiles.length > 0 && !sizeError,
        copiedImage,
        fileInputRef,
        imageChange,
        removeSelectedImage,
        handleUpload,
        handleDelete,
        handleCopy,
    };
};

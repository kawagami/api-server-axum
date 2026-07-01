import { useState, useRef } from 'react';
import { uploadImages } from '@/api/images';
import { deleteImage } from '@/api/images';

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
    const [copiedImage, setCopiedImage] = useState<string | null>(null);

    const fileInputRef = useRef<HTMLInputElement>(null);

    const imageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files?.length) {
            setSelectedFiles(Array.from(e.target.files));
        }
    };

    const removeSelectedImage = () => {
        setSelectedFiles([]);
        if (fileInputRef.current) fileInputRef.current.value = '';
    };

    const handleUpload = async () => {
        if (!selectedFiles.length) return;
        setIsUploading(true);
        try {
            const formData = new FormData();
            selectedFiles.forEach(f => formData.append('file', f));

            const responses = await uploadImages(formData);
            const newImages = responses.map(r => ({ name: r.id, url: r.url, status: r.status }));
            setImages((prev) => [...prev, ...newImages]);
            removeSelectedImage();
        } catch (err) {
            console.error('Upload error:', err);
        } finally {
            setIsUploading(false);
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
        copiedImage,
        fileInputRef,
        imageChange,
        removeSelectedImage,
        handleUpload,
        handleDelete,
        handleCopy,
    };
};

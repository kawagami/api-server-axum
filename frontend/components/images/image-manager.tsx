"use client";

import { usePathname } from 'next/navigation';
import { useImageManager, type ManagedImage } from '@/hooks/useImageManager';
import UploadSection from '@/components/images/upload-section';
import ImageGrid from '@/components/images/image-grid';

export default function ImageManager({ initialImages }: { initialImages: ManagedImage[] }) {
    const pathname = usePathname();
    const {
        images, deletingImage, selectedFiles, isUploading, copiedImage,
        fileInputRef, imageChange, removeSelectedImage, handleUpload, handleDelete, handleCopy,
    } = useImageManager(initialImages);

    return (
        <div className="container mx-auto p-6">
            {pathname === '/admin/images' && (
                <UploadSection
                    fileInputRef={fileInputRef}
                    selectedFiles={selectedFiles}
                    isUploading={isUploading}
                    onImageChange={imageChange}
                    onRemoveSelectedImage={removeSelectedImage}
                    onUpload={handleUpload}
                />
            )}
            <ImageGrid
                images={images}
                deletingImage={deletingImage}
                copiedImage={copiedImage}
                onDelete={handleDelete}
                onCopy={handleCopy}
            />
        </div>
    );
}

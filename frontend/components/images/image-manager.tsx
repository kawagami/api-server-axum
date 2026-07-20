"use client";

import { usePathname } from 'next/navigation';
import { useImageManager, type ManagedImage } from '@/hooks/useImageManager';
import UploadSection from '@/components/images/upload-section';
import ImageGrid from '@/components/images/image-grid';
import type { ImageCompressConfig } from '@/libs/image-config';

export default function ImageManager({ initialImages, compressConfig }: { initialImages: ManagedImage[]; compressConfig: ImageCompressConfig }) {
    const pathname = usePathname();
    const {
        images, deletingImage, selectedFiles, isUploading, uploadProgress, uploadError, canUpload, copiedImage,
        fileInputRef, imageChange, removeSelectedImage, handleUpload, handleDelete, handleCopy,
    } = useImageManager(initialImages, compressConfig);

    return (
        <div className="container mx-auto">
            {pathname === '/admin/images' && (
                <UploadSection
                    fileInputRef={fileInputRef}
                    selectedFiles={selectedFiles}
                    isUploading={isUploading}
                    uploadProgress={uploadProgress}
                    uploadError={uploadError}
                    canUpload={canUpload}
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

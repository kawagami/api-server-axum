"use client";

import { useMemo, useState } from 'react';
import { useImageManager, type ManagedImage } from '@/hooks/useImageManager';
import UploadSection from '@/components/images/upload-section';
import ImageGrid from '@/components/images/image-grid';
import DeleteConfirmModal from '@/components/images/delete-confirm-modal';
import PageHeader from '@/components/admin/page-header';
import type { ImageCompressConfig } from '@/libs/image-config';
import { formatBytes } from '@/libs/format-bytes';
import type { Image } from '@/types';

type Filter = 'all' | 'active' | 'unused';

export default function ImageManager({ initialImages, compressConfig }: { initialImages: Image[]; compressConfig: ImageCompressConfig }) {
    const {
        images, deletingImage, selectedFiles, isUploading, uploadProgress, uploadError, canUpload, copiedImage,
        fileInputRef, addFiles, imageChange, removeSelectedAt, removeSelectedImage, handleUpload, handleDelete, handleCopy,
    } = useImageManager(initialImages, compressConfig);

    const [filter, setFilter] = useState<Filter>('all');
    const [pendingDelete, setPendingDelete] = useState<ManagedImage | null>(null);

    const counts = useMemo(() => ({
        all: images.length,
        active: images.filter(i => i.status === 'active').length,
        unused: images.filter(i => i.status === 'unused').length,
    }), [images]);

    // 只加總已知大小（size_bytes 為 null 的只有檔案遺失的舊圖）
    const totalBytes = useMemo(() => {
        const sum = (list: ManagedImage[]) => list.reduce((acc, i) => acc + (i.size_bytes ?? 0), 0);
        return { all: sum(images), unused: sum(images.filter(i => i.status === 'unused')) };
    }, [images]);

    const visibleImages = useMemo(
        () => (filter === 'all' ? images : images.filter(i => i.status === filter)),
        [images, filter],
    );

    const tabs: { key: Filter; label: string }[] = [
        { key: 'all', label: '全部' },
        { key: 'active', label: '使用中' },
        { key: 'unused', label: '未使用' },
    ];

    const confirmDelete = async () => {
        if (!pendingDelete) return;
        await handleDelete(pendingDelete.id);
        setPendingDelete(null);
    };

    return (
        <div className="w-full flex flex-col gap-4">
            <PageHeader
                title="圖片"
                description="上傳後複製網址貼進文章；未被引用的圖片會由排程清除"
                actions={
                    <span className="text-sm text-neutral-500 dark:text-neutral-400 tabular-nums">
                        共 {counts.all} 張 · {formatBytes(totalBytes.all)}
                        {totalBytes.unused > 0 && `（未使用 ${formatBytes(totalBytes.unused)}）`}
                    </span>
                }
            />
            <UploadSection
                fileInputRef={fileInputRef}
                selectedFiles={selectedFiles}
                isUploading={isUploading}
                uploadProgress={uploadProgress}
                uploadError={uploadError}
                canUpload={canUpload}
                onAddFiles={addFiles}
                onImageChange={imageChange}
                onRemoveSelectedAt={removeSelectedAt}
                onRemoveSelectedImage={removeSelectedImage}
                onUpload={handleUpload}
            />

            {/* 篩選列 */}
            <div className="flex flex-wrap items-center gap-2">
                {tabs.map(tab => (
                    <button
                        key={tab.key}
                        onClick={() => setFilter(tab.key)}
                        className={`px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
                            filter === tab.key
                                ? 'bg-primary-600 text-white'
                                : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700'
                        }`}
                    >
                        {tab.label} <span className="opacity-70">{counts[tab.key]}</span>
                    </button>
                ))}
                <span className="ml-auto text-xs text-neutral-400">「未使用」未被任何內容引用，會由排程自動清除</span>
            </div>

            <ImageGrid
                images={visibleImages}
                copiedImage={copiedImage}
                onRequestDelete={setPendingDelete}
                onCopy={handleCopy}
                emptyHint={filter === 'all' ? '尚無圖片,拖放或點選上方區塊上傳' : '此分類沒有圖片'}
            />

            {pendingDelete && (
                <DeleteConfirmModal
                    image={pendingDelete}
                    deleting={deletingImage === pendingDelete.id}
                    onConfirm={confirmDelete}
                    onClose={() => setPendingDelete(null)}
                />
            )}
        </div>
    );
}

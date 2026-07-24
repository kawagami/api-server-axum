import Image from 'next/image';
import { Check, Copy, ImageOff, Trash2 } from 'lucide-react';
import type { ManagedImage } from '@/hooks/useImageManager';

interface Props {
    images: ManagedImage[];
    copiedImage: string | null;
    onRequestDelete: (image: ManagedImage) => void;
    onCopy: (url: string) => void;
    emptyHint?: string;
}

// active=使用中(primary)、unused=未使用(琥珀警示,將由排程清除)
const STATUS_META: Record<string, { label: string; className: string }> = {
    active: { label: '使用中', className: 'bg-primary-100 text-primary-700 dark:bg-primary-900/60 dark:text-primary-300' },
    unused: { label: '未使用', className: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/60 dark:text-yellow-300' },
};

const ImageGrid = ({ images, copiedImage, onRequestDelete, onCopy, emptyHint }: Props) => {
    if (images.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center gap-3 py-16 text-neutral-400">
                <ImageOff className="w-10 h-10" />
                <p className="text-sm">{emptyHint ?? '尚無圖片'}</p>
            </div>
        );
    }

    return (
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4 mt-6">
            {images.map((image) => {
                const status = image.status ? STATUS_META[image.status] : null;
                const copied = copiedImage === image.url;
                return (
                    <div
                        key={image.name}
                        className="group flex flex-col overflow-hidden rounded-xl bg-white dark:bg-neutral-800 ring-1 ring-neutral-200 dark:ring-neutral-700 hover:ring-primary-400 dark:hover:ring-primary-600 hover:shadow-md transition-shadow"
                    >
                        <div className="relative aspect-square overflow-hidden bg-neutral-100 dark:bg-neutral-900">
                            <Image
                                fill
                                sizes="(max-width: 640px) 50vw, (max-width: 1024px) 33vw, 25vw"
                                src={image.url}
                                alt={image.name}
                                className="object-cover transition-transform duration-300 group-hover:scale-105"
                            />
                            {status && (
                                <span className={`absolute top-2 left-2 text-xs font-semibold px-2 py-0.5 rounded-full ${status.className}`}>
                                    {status.label}
                                </span>
                            )}
                        </div>
                        <div className="flex items-center gap-1 p-2 border-t border-neutral-100 dark:border-neutral-700">
                            <button
                                onClick={() => onCopy(image.url)}
                                className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                                    copied
                                        ? 'text-primary-600 dark:text-primary-400'
                                        : 'text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700'
                                }`}
                            >
                                {copied ? <Check size={16} /> : <Copy size={16} />}
                                {copied ? '已複製' : '複製連結'}
                            </button>
                            <button
                                onClick={() => onRequestDelete(image)}
                                aria-label="刪除圖片"
                                className="p-2 rounded-lg text-neutral-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 transition-colors"
                            >
                                <Trash2 size={16} />
                            </button>
                        </div>
                    </div>
                );
            })}
        </div>
    );
};

export default ImageGrid;

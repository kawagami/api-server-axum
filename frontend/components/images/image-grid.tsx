import Image from 'next/image';
import { usePathname } from 'next/navigation';
import { Loader2 } from 'lucide-react';
import type { ManagedImage } from '@/hooks/useImageManager';

interface Props {
    images: ManagedImage[];
    deletingImage: string | null;
    copiedImage: string | null;
    onDelete: (name: string) => void;
    onCopy: (url: string) => void;
}

const ImageGrid = ({ images, deletingImage, copiedImage, onDelete, onCopy }: Props) => {
    const pathname = usePathname();

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-8">
            {images.map((image) => (
                <div key={image.name} className="bg-white dark:bg-neutral-800 p-4 rounded shadow-md flex flex-col items-center">
                    <Image width={150} height={150} src={image.url} alt={`Image ${image.name}`} className="rounded-lg mb-4 object-cover" />
                    {image.status && (
                        <span className={`text-xs font-semibold px-2 py-0.5 rounded mb-2 ${image.status === 'active' ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300'}`}>
                            {image.status}
                        </span>
                    )}
                    {pathname === '/admin/images' && (
                        <button
                            onClick={() => { if (window.confirm('確定刪除此圖片？')) onDelete(image.name); }}
                            className={`mt-2 py-1 px-4 rounded transition ${deletingImage === image.name ? 'bg-neutral-500 cursor-not-allowed' : 'bg-red-500 hover:bg-red-600 text-white'}`}
                            disabled={deletingImage === image.name}
                        >
                            {deletingImage === image.name ? (
                                <span className="flex items-center gap-1"><Loader2 className="w-4 h-4 animate-spin" />Deleting...</span>
                            ) : 'Delete'}
                        </button>
                    )}
                    <button onClick={() => onCopy(image.url)} className="mt-2 py-1 px-4 rounded bg-green-500 hover:bg-green-600 text-white">
                        Copy URL
                    </button>
                    {copiedImage === image.url && <span className="mt-2 text-sm text-green-500">URL 已複製！</span>}
                </div>
            ))}
        </div>
    );
};

export default ImageGrid;

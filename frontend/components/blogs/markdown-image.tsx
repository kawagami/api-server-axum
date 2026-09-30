import Image from 'next/image';
import type { ImagePlaceholder } from '@/types';

/**
 * markdown 內文的 `<img>`（server / client component 皆可用）。
 *
 * 站內圖有後端下發的版位資訊時：用原圖真實寬高預留版位（免 CLS），並以 `placeholder="blur"`
 * 在原圖載入前顯示模糊小圖。`sizes` 必帶 —— 沒有它 next/image 會依 `width` 出 1x/2x srcset，
 * 寬 4000 的原圖在 1x 就會抓 3840w，比內文欄寬大好幾倍。
 * 外部圖沒有版位資訊，維持原本的 800×600 + auto 尺寸。
 */
export default function MarkdownImage({ src, alt, images }: {
    src?: string | Blob;
    alt?: string;
    images?: Record<string, ImagePlaceholder>;
}) {
    const url = typeof src === 'string' ? src : '';
    const meta = images?.[url];

    if (!meta) {
        return (
            <Image
                src={url}
                alt={alt || ''}
                width={800}
                height={600}
                style={{ width: 'auto', height: 'auto', maxWidth: '100%' }}
            />
        );
    }

    return (
        <Image
            src={url}
            alt={alt || ''}
            width={meta.width}
            height={meta.height}
            sizes="(max-width: 768px) 100vw, 768px"
            placeholder={meta.blur_data_url ? 'blur' : 'empty'}
            blurDataURL={meta.blur_data_url ?? undefined}
            style={{ maxWidth: '100%', height: 'auto' }}
        />
    );
}

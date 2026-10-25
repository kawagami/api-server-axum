import { getImages } from "@/api/images";
import { getPublicSettings } from "@/api/settings";
import ImageManager from "@/components/images/image-manager";
import { requirePermission } from "@/libs/admin-permissions";
import { resolveImageCompressConfig } from "@/libs/image-config";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "圖片",
    description: "上傳與管理文章用圖片",
};

export default async function Images() {
    await requirePermission("image:read");
    const [images, publicSettings] = await Promise.all([getImages(), getPublicSettings()]);

    return (
        <div className="w-full">
            <ImageManager initialImages={images} compressConfig={resolveImageCompressConfig(publicSettings)} />
        </div>
    );
}

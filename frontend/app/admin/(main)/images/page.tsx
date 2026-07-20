import { getImages } from "@/api/images";
import { getPublicSettings } from "@/api/settings";
import ImageManager from "@/components/images/image-manager";
import { requirePermission } from "@/libs/admin-permissions";
import { resolveImageCompressConfig } from "@/libs/image-config";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Images page",
    description: "Images page",
};

export default async function Images() {
    await requirePermission("image:read");
    const [images, publicSettings] = await Promise.all([getImages(), getPublicSettings()]);
    const managedImages = images.map(img => ({ name: img.id, url: img.url, status: img.status }));

    return (
        <div className="w-full">
            <ImageManager initialImages={managedImages} compressConfig={resolveImageCompressConfig(publicSettings)} />
        </div>
    );
}

import { getImages } from "@/api/images";
import ImageManager from "@/components/images/image-manager";
import { requirePermission } from "@/libs/admin-permissions";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Images page",
    description: "Images page",
};

export default async function Images() {
    await requirePermission("image:read");
    const images = await getImages();
    const managedImages = images.map(img => ({ name: img.id, url: img.url, status: img.status }));

    return (
        <div className="w-full">
            <ImageManager initialImages={managedImages} />
        </div>
    );
}

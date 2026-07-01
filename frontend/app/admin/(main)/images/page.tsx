import { getImages } from "@/api/images";
import ImageManager from "@/components/images/image-manager";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Images page",
    description: "Images page",
};

export default async function Images() {
    const images = await getImages();
    const managedImages = images.map(img => ({ name: img.id, url: img.url, status: img.status }));

    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-3 sm:p-6">
            <ImageManager initialImages={managedImages} />
        </div>
    );
}

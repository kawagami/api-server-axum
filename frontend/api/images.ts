"use server";

import adminRequest from "@/libs/adminRequest";
import type { Image } from "@/types";

export async function getImages(): Promise<Image[]> {
    return adminRequest<Image[]>({
        url: `${process.env.API_URL}/admin/images`,
    });
}

export async function uploadImages(formData: FormData): Promise<Image[]> {
    return adminRequest<Image[]>({
        url: `${process.env.API_URL}/admin/images/upload_multiple`,
        method: 'POST',
        body: formData,
    });
}

export async function deleteImage(id: string): Promise<void> {
    await adminRequest({
        url: `${process.env.API_URL}/admin/images/${id}`,
        method: 'DELETE',
    });
}

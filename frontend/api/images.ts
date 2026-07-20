"use server";

import adminRequest from "@/libs/adminRequest";
import type { Image } from "@/types";

export async function getImages(): Promise<Image[]> {
    return adminRequest<Image[]>({
        url: `${process.env.API_URL}/admin/images`,
    });
}

export async function uploadImage(formData: FormData): Promise<Image> {
    return adminRequest<Image>({
        url: `${process.env.API_URL}/admin/images`,
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

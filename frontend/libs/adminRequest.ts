"use server";

import { redirect } from 'next/navigation';
import { headers as nextHeaders } from "next/headers";
import { createAuthRequest } from "@/libs/createAuthRequest";

const adminRequest = createAuthRequest("session", async () => {
    const headersList = await nextHeaders();
    const referer = headersList.get('referer') || '';
    let redirectPath = '/admin';
    try {
        if (referer) redirectPath = new URL(referer).pathname;
    } catch { /* ignore invalid referer */ }
    redirect(`/admin/login?redirect=${encodeURIComponent(redirectPath)}`);
});

export default adminRequest;

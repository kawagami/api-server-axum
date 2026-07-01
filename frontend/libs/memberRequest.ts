"use server";

import { getLocale } from 'next-intl/server';
import { redirect } from 'next/navigation';
import { createAuthRequest } from "@/libs/createAuthRequest";

const memberRequest = createAuthRequest("access_token", async () => {
    const locale = await getLocale();
    redirect(`/${locale}/login`);
});

export default memberRequest;

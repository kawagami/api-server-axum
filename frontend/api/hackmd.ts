"use server";

import { fetchApi } from "@/libs/fetchApi";
import type { HackmdNote, HackmdTag } from "@/types";

export async function getHackMDNotes(): Promise<HackmdNote[]> {
    return fetchApi(`${process.env.API_URL}/notes/lists`, { next: { revalidate: 60 } });
}

export async function getHackMDNoteTags(): Promise<HackmdTag[]> {
    return fetchApi(`${process.env.API_URL}/notes/tags`, { next: { revalidate: 60 } });
}

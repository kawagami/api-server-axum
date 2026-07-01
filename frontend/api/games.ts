"use server";

import adminRequest from "@/libs/adminRequest";
import type { GameOverview } from "@/types";

export async function getGamesOverview(): Promise<GameOverview[]> {
    return adminRequest<GameOverview[]>({
        url: `${process.env.API_URL}/admin/games`,
    });
}

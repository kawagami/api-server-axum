import { getGamesOverview } from "@/api/games";
import GamesOverview from "./games-overview";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "對局總覽",
    description: "即時各款遊戲對局狀況",
};

export default async function GamesOverviewPage() {
    const initial = await getGamesOverview();

    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-6">
            <div className="max-w-4xl mx-auto">
                <GamesOverview initial={initial} />
            </div>
        </div>
    );
}

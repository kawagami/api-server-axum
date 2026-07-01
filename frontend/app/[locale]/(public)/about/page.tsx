import IndexInfo from "@/components/index-info";
import { getTranslations } from "next-intl/server";

const CARD_HREFS = ["/", "/hackmd-notes", "/tools/countdown"];

export default async function About() {
    const t = await getTranslations("About");
    const cards = t.raw("cards") as { title: string; contents: string[] }[];

    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-center w-full px-4">
            {cards.map((card, index) => (
                <IndexInfo
                    key={`index_info_${index}`}
                    href={CARD_HREFS[index] ?? "/"}
                    title={card.title}
                    contents={card.contents}
                    introLabel={t("intro")}
                />
            ))}
        </div>
    );
}

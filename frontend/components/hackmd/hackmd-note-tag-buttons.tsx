"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import type { HackmdTag } from "@/types";

interface Props {
    tags: HackmdTag[];
    openArray: string[];
    setOpenArray: (arr: string[]) => void;
}

export default function HackmdNoteTagButtons({ tags, openArray, setOpenArray }: Props) {
    const t = useTranslations("Hackmd");
    const [showFilter, setShowFilter] = useState(false);

    const checkExist = (name: string) => {
        const result = openArray.includes(name)
            ? openArray.filter(o => o !== name)
            : [name, ...openArray];
        setOpenArray(result);
    };

    const commonStyle = "p-2 border-2 rounded-lg bg-neutral-200 dark:bg-neutral-800 dark:text-white hover:border-primary-400 transition-colors";

    return (
        <>
            <div className="gap-1 text-center p-2 mb-2">
                <button
                    type="button"
                    className={commonStyle}
                    aria-expanded={showFilter}
                    onClick={() => setShowFilter(!showFilter)}
                >
                    {showFilter ? t("hideFilter") : t("showFilter")}
                </button>
            </div>
            {showFilter && (
                <div>
                    <div className="flex flex-wrap justify-center gap-2 p-2 mb-2">
                        <button type="button" className={commonStyle} onClick={() => setOpenArray(tags.map(tag => tag.name))}>{t("selectAll")}</button>
                        <button type="button" className={commonStyle} onClick={() => setOpenArray([])}>{t("deselectAll")}</button>
                    </div>
                    <div className="flex flex-wrap justify-center gap-2">
                        {tags.map(tag => {
                            const active = openArray.includes(tag.name);
                            const tailwindStyle = active
                                ? "bg-primary-500 border-primary-500 text-white hover:bg-primary-400 hover:border-primary-400"
                                : "bg-neutral-300 dark:bg-neutral-600 hover:bg-primary-100 dark:hover:bg-primary-900";
                            return (
                                <button
                                    type="button"
                                    className={`${commonStyle} ${tailwindStyle}`}
                                    key={tag.id}
                                    aria-pressed={active}
                                    onClick={() => checkExist(tag.name)}
                                >
                                    {tag.name}
                                </button>
                            );
                        })}
                    </div>
                </div>
            )}
        </>
    );
}

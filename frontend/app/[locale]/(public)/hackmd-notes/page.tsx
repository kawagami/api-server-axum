import { getHackMDNotes, getHackMDNoteTags } from "@/api/hackmd";
import HackMDNotesWrapper from "@/components/hackmd/hackmd-notes-wrapper";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "HackMD Notes",
    description: "HackMD Notes",
};

export default async function HackMDNotes() {
    // ISR 頁：build 階段（Docker 內無 env / 後端不可達）fetch 失敗時以空資料預渲染，
    // runtime 第一次 revalidate 就會補上真資料
    const [notes, tags] = await Promise.all([
        getHackMDNotes().catch(() => []),
        getHackMDNoteTags().catch(() => []),
    ]);

    return (
        <div className="w-full h-[calc(100svh-120px)] text-center overflow-auto">
            <HackMDNotesWrapper notes={notes} tags={tags} />
        </div>
    );
}

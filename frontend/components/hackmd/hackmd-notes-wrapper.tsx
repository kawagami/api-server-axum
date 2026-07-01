"use client";

import { useState } from "react";
import HackmdNoteTagButtons from "./hackmd-note-tag-buttons";
import HackMDNotesComponent from "./hackmd-notes";
import type { HackmdNote, HackmdTag } from "@/types";

interface Props {
    notes: HackmdNote[];
    tags: HackmdTag[];
}

export default function HackMDNotesWrapper({ notes, tags }: Props) {
    const [openArray, setOpenArray] = useState<string[]>(tags.map(t => t.name));

    return (
        <>
            <HackmdNoteTagButtons tags={tags} openArray={openArray} setOpenArray={setOpenArray} />
            <HackMDNotesComponent notes={notes} openArray={openArray} />
        </>
    );
}

"use client";

import { useActionState } from "react";
import { Loader2 } from "lucide-react";
import { saySomethingToSomeone, type SaySomethingResult } from "./actions";

const initialState: SaySomethingResult = { ok: false };

export default function SaySomethingForm() {
    const [state, formAction, isPending] = useActionState(saySomethingToSomeone, initialState);

    return (
        <form action={formAction} className="flex flex-col gap-3">
            <div className="flex flex-col gap-1">
                <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">Addr</label>
                <input
                    name="addr"
                    type="text"
                    placeholder="socket addr (e.g. 1.2.3.4:5678)"
                    className="border border-neutral-300 dark:border-neutral-600 rounded px-3 py-2 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 text-sm"
                    required
                />
            </div>
            <div className="flex flex-col gap-1">
                <label className="text-sm font-medium text-neutral-700 dark:text-neutral-300">Message</label>
                <input
                    name="message"
                    type="text"
                    placeholder="message content"
                    className="border border-neutral-300 dark:border-neutral-600 rounded px-3 py-2 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 text-sm"
                    required
                />
            </div>
            <button
                type="submit"
                disabled={isPending}
                className="self-start px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white text-sm font-medium rounded"
            >
                {isPending ? (
                    <span className="flex items-center gap-1"><Loader2 className="w-4 h-4 animate-spin" />Sending…</span>
                ) : "Send"}
            </button>
            {state.message && (
                <p className={`text-sm ${state.ok ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"}`}>
                    {state.message}
                </p>
            )}
        </form>
    );
}

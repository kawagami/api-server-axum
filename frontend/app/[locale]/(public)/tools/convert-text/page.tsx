"use client";

import { useState, useEffect, useActionState } from "react";
import { convertTextAction } from "@/app/[locale]/(public)/tools/convert-text/actions";

interface Notification {
    type: string;
    message: string;
}

const DIRECTIONS = [
    { value: "s2t", label: "簡→繁" },
    { value: "t2s", label: "繁→簡" },
] as const;

export default function ConvertText() {
    const initialState = { status: null, message: null, converted_text: '' };
    const [state, formAction, isPending] = useActionState(convertTextAction, initialState);
    const [notification, setNotification] = useState<Notification | null>(null);
    const [direction, setDirection] = useState<"s2t" | "t2s">("s2t");

    const handleCopy = async () => {
        if (!state.converted_text.trim()) {
            setNotification({ type: "error", message: "沒有可以複製的文本" });
            return;
        }
        try {
            await navigator.clipboard.writeText(state.converted_text);
            setNotification({ type: "success", message: "文本已成功複製到剪貼板" });
        } catch {
            setNotification({ type: "error", message: "複製失敗，請手動嘗試" });
        }
    };

    // action state 變更時同步 notification（adjust-state-during-render 模式，取代 useEffect）
    const [prevState, setPrevState] = useState(state);
    if (prevState !== state) {
        setPrevState(state);
        if (state.status && state.message) {
            setNotification({ type: state.status, message: state.message });
        }
    }

    useEffect(() => {
        if (notification) {
            const timer = setTimeout(() => setNotification(null), 2000);
            return () => clearTimeout(timer);
        }
    }, [notification]);

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto flex flex-col items-center">
            <h1 className="text-2xl font-bold mb-4">中文轉換</h1>
            <form action={formAction} className="w-4/6 flex flex-col items-center">
                <input type="hidden" name="direction" value={direction} />
                <div className="flex gap-2 mb-3">
                    {DIRECTIONS.map(({ value, label }) => (
                        <button
                            key={value}
                            type="button"
                            onClick={() => setDirection(value)}
                            className={`px-4 py-1.5 rounded-md border text-sm font-medium transition-colors ${direction === value ? "bg-primary-500 text-white border-primary-500" : "border-neutral-300 hover:border-primary-400 dark:border-neutral-600"}`}
                        >
                            {label}
                        </button>
                    ))}
                </div>
                <textarea
                    name="inputText"
                    rows={5}
                    placeholder={direction === "s2t" ? "請輸入簡體中文" : "請輸入繁體中文"}
                    className="w-full mb-3 p-3 text-lg border border-neutral-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 dark:bg-neutral-800 dark:border-neutral-600"
                />
                <button
                    type="submit"
                    className="px-5 py-2 text-lg bg-primary-500 text-white rounded-md hover:bg-primary-600 focus:outline-none focus:ring-2 focus:ring-primary-400 mb-3"
                    disabled={isPending}
                >
                    {isPending ? "處理中..." : "轉換"}
                </button>
            </form>
            <h2 className="text-xl font-semibold mb-2">轉換結果：</h2>
            <textarea
                rows={5}
                readOnly
                value={state.converted_text}
                className="w-4/6 p-3 text-lg border border-neutral-300 rounded-md bg-neutral-100 dark:bg-neutral-700 dark:border-neutral-600"
            />
            <button
                onClick={handleCopy}
                className="px-5 py-2 text-lg bg-green-500 text-white rounded-md hover:bg-green-600 focus:outline-none focus:ring-2 focus:ring-green-400 mt-3"
            >
                複製結果
            </button>
            {notification && (
                <div className={`fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 px-4 py-2 rounded-md shadow-lg text-white ${notification.type === "success" ? "bg-green-500" : "bg-red-500"}`}>
                    {notification.message}
                </div>
            )}
        </div>
    );
}

"use client";

import { useActionState, useEffect, useRef } from "react";
import { Loader2, Send } from "lucide-react";
import { saySomethingToSomeone, type SaySomethingResult } from "./actions";

const initialState: SaySomethingResult = { ok: false };

interface Props {
    // 目前選定的目標連線；null = 還沒從列表選
    addr: string | null;
    // 選定的連線是否還在線上（斷線後仍留著選取狀態，要提示）
    online: boolean;
    // 每次從列表點「發訊息」都會遞增，用來把焦點移到訊息框
    focusToken: number;
}

export default function SaySomethingForm({ addr, online, focusToken }: Props) {
    const [state, formAction, isPending] = useActionState(saySomethingToSomeone, initialState);
    const formRef = useRef<HTMLFormElement>(null);
    const messageRef = useRef<HTMLTextAreaElement>(null);

    // 送出成功就清空訊息框，避免同一則被重複送出。
    // 每次 action 回傳都是新物件，所以同一個成功結果只會清一次。
    useEffect(() => {
        if (state.ok) formRef.current?.reset();
    }, [state]);

    // focusToken 初值 0 不搶焦點，只有實際點過列表按鈕才移動
    useEffect(() => {
        if (focusToken > 0) messageRef.current?.focus();
    }, [focusToken]);

    if (!addr) {
        return (
            <p className="text-sm text-neutral-500 dark:text-neutral-400">
                請先從上方連線列表點「發訊息」選一個目標。
            </p>
        );
    }

    return (
        <form ref={formRef} action={formAction} className="flex flex-col gap-3">
            <input type="hidden" name="addr" value={addr} />
            <div className="flex flex-col gap-1">
                <span className="text-sm font-medium text-neutral-700 dark:text-neutral-300">目標連線</span>
                <div className="flex items-center gap-2 flex-wrap">
                    <code className="font-mono text-sm px-2 py-1 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-800 dark:text-neutral-200 break-all">
                        {addr}
                    </code>
                    {!online && (
                        <span className="text-xs text-amber-600 dark:text-amber-400">
                            此連線已不在線上，送出會失敗
                        </span>
                    )}
                </div>
            </div>
            <div className="flex flex-col gap-1">
                <label htmlFor="ws-message" className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                    訊息內容
                </label>
                <textarea
                    id="ws-message"
                    ref={messageRef}
                    name="message"
                    rows={3}
                    placeholder="會以 admin_message 事件推給該連線"
                    className="border border-neutral-300 dark:border-neutral-600 rounded px-3 py-2 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 text-sm resize-y"
                    required
                />
            </div>
            <button
                type="submit"
                disabled={isPending}
                className="self-start flex items-center gap-1.5 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white text-sm font-medium rounded transition-colors"
            >
                {isPending
                    ? <Loader2 className="w-4 h-4 animate-spin" />
                    : <Send className="w-4 h-4" />}
                {isPending ? "送出中…" : "送出"}
            </button>
            {state.message && (
                <p
                    role="status"
                    className={`text-sm ${state.ok ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"}`}
                >
                    {state.message}
                </p>
            )}
        </form>
    );
}

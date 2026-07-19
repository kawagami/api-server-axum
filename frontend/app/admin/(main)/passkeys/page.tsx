"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { KeyRound, Loader2, Trash2 } from "lucide-react";
import { startRegistration } from "@simplewebauthn/browser";
import {
    beginPasskeyRegistration,
    deletePasskey,
    finishPasskeyRegistration,
    getPasskeys,
} from "@/api/auth";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";
import type { PasskeyItem } from "@/types";

function formatTime(value: string | null): string {
    if (!value) return "—";
    return new Date(value).toLocaleString("zh-TW", { timeZone: "Asia/Taipei", hour12: false });
}

export default function PasskeysPage() {
    const [passkeys, setPasskeys] = useState<PasskeyItem[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [creating, setCreating] = useState(false);
    const [deletingId, setDeletingId] = useState<number | null>(null);
    const labelRef = useRef<HTMLInputElement>(null);

    const reload = useCallback(async () => {
        try {
            setPasskeys(await getPasskeys());
        } catch {
            setError("載入失敗，請重新整理");
        }
    }, []);

    useEffect(() => {
        // setState 發生在 fetch await 之後，非同步、不會 cascading render
        // eslint-disable-next-line react-hooks/set-state-in-effect
        reload();
    }, [reload]);

    const handleCreate = async () => {
        setError(null);
        setCreating(true);
        try {
            const options = await beginPasskeyRegistration();
            const credential = await startRegistration({ optionsJSON: options.publicKey });
            const label = labelRef.current?.value.trim() || "我的裝置";
            await finishPasskeyRegistration(credential, label);
            if (labelRef.current) labelRef.current.value = "";
            await reload();
        } catch (e) {
            const err = e as Error & { status?: number };
            if (err.name === "NotAllowedError" || err.name === "AbortError") {
                // 使用者取消，不當錯誤
            } else if (err.status === 409) {
                setError("此裝置已註冊過 passkey");
            } else if (err.name === "InvalidStateError") {
                setError("此裝置已註冊過 passkey");
            } else {
                setError("建立失敗，請確認裝置支援 passkey 後再試");
            }
        } finally {
            setCreating(false);
        }
    };

    const handleDelete = async (item: PasskeyItem) => {
        if (!confirm(`確定刪除 passkey「${item.label}」？刪除後此裝置需改用密碼登入。`)) return;
        setError(null);
        setDeletingId(item.id);
        try {
            await deletePasskey(item.id);
            await reload();
        } catch {
            setError("刪除失敗，請稍後再試");
        } finally {
            setDeletingId(null);
        }
    };

    return (
        <div className="max-w-3xl space-y-6">
            <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">Passkey 管理</h1>
            <p className="text-sm text-neutral-600 dark:text-neutral-400">
                Passkey 讓你以指紋、臉部辨識或裝置密碼登入後台，免輸入密碼。密碼登入永遠保留，刪光 passkey 也不會被鎖在門外。
            </p>

            <div className="flex flex-col sm:flex-row gap-2">
                <input
                    type="text"
                    ref={labelRef}
                    placeholder="名稱（如：工作筆電）"
                    maxLength={64}
                    className="flex-1 px-3 py-2 rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
                <button
                    type="button"
                    onClick={handleCreate}
                    disabled={creating}
                    className="px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors"
                >
                    {creating ? (
                        <span className="flex items-center justify-center gap-2"><Loader2 className="w-4 h-4 animate-spin" />建立中...</span>
                    ) : (
                        <span className="flex items-center justify-center gap-2"><KeyRound className="w-4 h-4" />新增 Passkey</span>
                    )}
                </button>
            </div>

            {error && <p className="text-sm text-red-500">{error}</p>}

            {passkeys === null ? (
                <div className="flex items-center gap-2 text-neutral-500"><Loader2 className="w-4 h-4 animate-spin" />載入中...</div>
            ) : passkeys.length === 0 ? (
                <p className="text-sm text-neutral-500">尚未建立任何 passkey。</p>
            ) : (
                <div className="overflow-x-auto">
                    <AdminTable>
                        <thead>
                            <AdminHeadRow>
                                <AdminTh>名稱</AdminTh>
                                <AdminTh>建立時間</AdminTh>
                                <AdminTh>最後使用</AdminTh>
                                <AdminTh className="w-16">操作</AdminTh>
                            </AdminHeadRow>
                        </thead>
                        <tbody>
                            {passkeys.map((item) => (
                                <AdminRow key={item.id}>
                                    <AdminTd>{item.label}</AdminTd>
                                    <AdminTd>{formatTime(item.created_at)}</AdminTd>
                                    <AdminTd>{formatTime(item.last_used_at)}</AdminTd>
                                    <AdminTd>
                                        <button
                                            type="button"
                                            onClick={() => handleDelete(item)}
                                            disabled={deletingId === item.id}
                                            className="text-red-500 hover:text-red-600 disabled:opacity-50"
                                            title="刪除"
                                        >
                                            {deletingId === item.id ? (
                                                <Loader2 className="w-4 h-4 animate-spin" />
                                            ) : (
                                                <Trash2 className="w-4 h-4" />
                                            )}
                                        </button>
                                    </AdminTd>
                                </AdminRow>
                            ))}
                        </tbody>
                    </AdminTable>
                </div>
            )}
        </div>
    );
}

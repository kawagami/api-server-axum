"use client";

import { useState } from "react";
import { postChangePassword } from "@/api/auth";

export default function ChangePasswordPage() {
    const [currentPassword, setCurrentPassword] = useState("");
    const [newPassword, setNewPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
    const [errorMsg, setErrorMsg] = useState("");

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setErrorMsg("");

        if (newPassword !== confirmPassword) {
            setErrorMsg("新密碼與確認密碼不一致");
            return;
        }

        setStatus("loading");
        try {
            await postChangePassword({ current_password: currentPassword, new_password: newPassword });
            setStatus("success");
            setCurrentPassword("");
            setNewPassword("");
            setConfirmPassword("");
        } catch (e) {
            const err = e as Error & { status?: number };
            setErrorMsg(err.status === 401 ? "舊密碼錯誤或 token 無效" : err.message);
            setStatus("error");
        }
    };

    const inputClass = "w-full px-3 py-2 rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-500";

    return (
        <div className="max-w-md">
            <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100 mb-6">修改密碼</h1>
            <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                    <label className="block text-sm text-neutral-600 dark:text-neutral-400 mb-1">目前密碼</label>
                    <input
                        type="password"
                        required
                        value={currentPassword}
                        onChange={e => setCurrentPassword(e.target.value)}
                        className={inputClass}
                    />
                </div>
                <div>
                    <label className="block text-sm text-neutral-600 dark:text-neutral-400 mb-1">新密碼</label>
                    <input
                        type="password"
                        required
                        value={newPassword}
                        onChange={e => setNewPassword(e.target.value)}
                        className={inputClass}
                    />
                </div>
                <div>
                    <label className="block text-sm text-neutral-600 dark:text-neutral-400 mb-1">確認新密碼</label>
                    <input
                        type="password"
                        required
                        value={confirmPassword}
                        onChange={e => setConfirmPassword(e.target.value)}
                        className={inputClass}
                    />
                </div>

                {status === "success" && (
                    <p className="text-sm text-green-600 dark:text-green-400">密碼已成功變更</p>
                )}
                {errorMsg && (
                    <p className="text-sm text-red-500">{errorMsg}</p>
                )}

                <button
                    type="submit"
                    disabled={status === "loading"}
                    className="w-full py-2 px-4 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors"
                >
                    {status === "loading" ? "處理中..." : "變更密碼"}
                </button>
            </form>
        </div>
    );
}

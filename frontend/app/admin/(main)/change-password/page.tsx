"use client";

import { useActionState, useEffect, useRef } from "react";
import { postChangePassword } from "@/api/auth";
import PageHeader from "@/components/admin/page-header";

type ChangeState = { error: string | null; success: boolean };

export default function ChangePasswordPage() {
    const formRef = useRef<HTMLFormElement>(null);

    const [state, formAction, pending] = useActionState<ChangeState, FormData>(
        async (_prev, formData) => {
            const currentPassword = String(formData.get("currentPassword") ?? "");
            const newPassword = String(formData.get("newPassword") ?? "");
            const confirmPassword = String(formData.get("confirmPassword") ?? "");

            if (newPassword !== confirmPassword) {
                return { error: "新密碼與確認密碼不一致", success: false };
            }

            try {
                await postChangePassword({ current_password: currentPassword, new_password: newPassword });
                return { error: null, success: true };
            } catch (e) {
                const err = e as Error & { status?: number };
                return { error: err.status === 401 ? "舊密碼錯誤或 token 無效" : err.message, success: false };
            }
        },
        { error: null, success: false }
    );

    // 成功後清空表單（uncontrolled inputs，靠 form.reset()）並導回登入頁。
    //
    // 後端改密碼成功會撤銷這個帳號的 session（清 Redis user:login:{id}），所以現在這張
    // token 已經失效了。不主動導頁的話，使用者會看到「成功」然後在下一個動作才被彈回
    // 登入頁，看起來像壞掉。延遲一下讓成功訊息看得到。
    useEffect(() => {
        if (!state.success) return;
        formRef.current?.reset();
        const timer = setTimeout(() => {
            window.location.href = "/admin/login?redirect=/admin";
        }, 1500);
        return () => clearTimeout(timer);
    }, [state.success]);

    // 焦點樣式由 globals.css 的全站 :focus-visible 規則統一提供，元件不再各自寫 focus 類別
    const inputClass = "w-full px-3 py-2 rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100";

    return (
        <div className="w-full max-w-2xl flex flex-col gap-4">
            <PageHeader title="修改密碼" />
            <form ref={formRef} action={formAction} className="space-y-4">
                <div>
                    <label className="block text-sm text-neutral-600 dark:text-neutral-400 mb-1">目前密碼</label>
                    <input
                        type="password"
                        name="currentPassword"
                        required
                        autoComplete="current-password"
                        className={inputClass}
                    />
                </div>
                <div>
                    <label className="block text-sm text-neutral-600 dark:text-neutral-400 mb-1">新密碼</label>
                    <input
                        type="password"
                        name="newPassword"
                        required
                        autoComplete="new-password"
                        className={inputClass}
                    />
                </div>
                <div>
                    <label className="block text-sm text-neutral-600 dark:text-neutral-400 mb-1">確認新密碼</label>
                    <input
                        type="password"
                        name="confirmPassword"
                        required
                        autoComplete="new-password"
                        className={inputClass}
                    />
                </div>

                {state.success && (
                    <p className="text-sm text-green-600 dark:text-green-400">
                        密碼已成功變更，即將導向登入頁，請以新密碼重新登入
                    </p>
                )}
                {state.error && (
                    <p className="text-sm text-red-500">{state.error}</p>
                )}

                <button
                    type="submit"
                    disabled={pending}
                    className="w-full py-2 px-4 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors"
                >
                    {pending ? "處理中…" : "變更密碼"}
                </button>
            </form>
        </div>
    );
}

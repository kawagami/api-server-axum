"use client";

import { useActionState, useEffect, useRef } from "react";
import { useTranslations } from "next-intl";
import { Send } from "lucide-react";
import { submitMessageAction, type ContactFormState } from "./actions";

const initialState: ContactFormState = { status: null, messageKey: null };

export default function ContactForm() {
    const t = useTranslations("Contact");
    const [state, formAction, isPending] = useActionState(submitMessageAction, initialState);
    const formRef = useRef<HTMLFormElement>(null);

    // 送出成功後清空表單
    useEffect(() => {
        if (state.status === "success") formRef.current?.reset();
    }, [state]);

    const inputClass =
        "w-full px-3 py-2 rounded-md border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-2 focus:ring-primary-400 transition-colors";

    return (
        <form ref={formRef} action={formAction} className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
                <label htmlFor="name" className="text-sm text-neutral-600 dark:text-neutral-300">
                    {t("nameLabel")}
                </label>
                <input id="name" name="name" type="text" maxLength={100}
                    placeholder={t("namePlaceholder")} className={inputClass} />
            </div>

            <div className="flex flex-col gap-1">
                <label htmlFor="email" className="text-sm text-neutral-600 dark:text-neutral-300">
                    {t("emailLabel")}
                </label>
                <input id="email" name="email" type="email" maxLength={200}
                    placeholder={t("emailPlaceholder")} className={inputClass} />
                <span className="text-xs text-neutral-400 dark:text-neutral-500">{t("emailHint")}</span>
            </div>

            <div className="flex flex-col gap-1">
                <label htmlFor="content" className="text-sm text-neutral-600 dark:text-neutral-300">
                    {t("contentLabel")}
                </label>
                <textarea id="content" name="content" rows={6} maxLength={5000} required
                    placeholder={t("contentPlaceholder")} className={`${inputClass} resize-y`} />
            </div>

            <button type="submit" disabled={isPending}
                className="inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-md bg-primary-600 hover:bg-primary-700 text-white font-medium disabled:opacity-50 transition-colors">
                <Send className="w-4 h-4" />
                {isPending ? t("submitting") : t("submit")}
            </button>

            {state.status && state.messageKey && (
                <p className={`text-sm text-center ${state.status === "success" ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"}`}>
                    {t(`result.${state.messageKey}`)}
                </p>
            )}
        </form>
    );
}

"use client";

import { useId, useState } from "react";
import { useTranslations } from "next-intl";
import { Trash2 } from "lucide-react";
import Modal from "@/components/modal";
import { cn } from "@/libs/cn";
import { PUBLIC_INPUT } from "@/libs/input-styles";
import type { FoodLogEntryInput, Meal } from "@/types";
import { MEALS } from "./model";
import { RatingInput } from "./rating";

/** 新增時帶入的值（日期 / 餐別沿用快速記錄區的選擇，品項帶搜尋框的字） */
export interface EntryFormInitial {
    eaten_on: string;
    meal: Meal;
    item: string;
    meal_label?: string | null;
    store?: string | null;
    quantity?: number;
    amount?: number | null;
    rating?: number | null;
    note?: string | null;
}

interface Props {
    initial: EntryFormInitial;
    isEdit: boolean;
    today: string;
    stores: string[];
    itemsByStore: Record<string, string[]>;
    onSubmit: (input: FoodLogEntryInput) => Promise<boolean>;
    onDelete?: () => Promise<boolean>;
    onClose: () => void;
}

// 上限與後端 structs/food_log.rs 一致；後端才是權威（超過回 422），這裡只是不讓人打超過
const MAX_ITEM = 100;
const MAX_STORE = 100;
const MAX_MEAL_LABEL = 50;
const MAX_NOTE = 1000;
const MAX_QUANTITY = 99;
const MAX_AMOUNT = 1_000_000;

const LABEL = "text-sm font-medium";

export default function EntryForm({ initial, isEdit, today, stores, itemsByStore, onSubmit, onDelete, onClose }: Props) {
    const t = useTranslations("FoodLog");
    const id = useId();
    const [eatenOn, setEatenOn] = useState(initial.eaten_on);
    const [meal, setMeal] = useState<Meal>(initial.meal);
    const [mealLabel, setMealLabel] = useState(initial.meal_label ?? "");
    const [store, setStore] = useState(initial.store ?? "");
    const [item, setItem] = useState(initial.item);
    const [quantity, setQuantity] = useState(String(initial.quantity ?? 1));
    const [amount, setAmount] = useState(initial.amount == null ? "" : String(initial.amount));
    const [rating, setRating] = useState<number | null>(initial.rating ?? null);
    const [note, setNote] = useState(initial.note ?? "");
    const [busy, setBusy] = useState(false);

    // 品項建議：選了店家就只列那家點過的，沒選就列全部
    const itemOptions = store.trim()
        ? itemsByStore[store.trim()] ?? []
        : [...new Set(Object.values(itemsByStore).flat())];

    async function run(action: () => Promise<boolean>) {
        setBusy(true);
        try {
            await action();
        } finally {
            setBusy(false);
        }
    }

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        run(() => onSubmit({
            eaten_on: eatenOn,
            meal,
            meal_label: mealLabel,
            store,
            item,
            quantity: Number(quantity) || 1,
            amount: amount === "" ? null : Number(amount),
            rating,
            note,
        }));
    }

    const title = isEdit ? t("editEntry") : t("newEntry");

    return (
        <Modal label={title} onClose={onClose} dismissible={!busy} size="lg" surface="public" className="p-5 max-h-[90svh] overflow-auto">
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                <h2 className="font-semibold">{title}</h2>

                <div className="grid grid-cols-2 gap-3">
                    <Field label={t("date")} htmlFor={`${id}-date`}>
                        <input id={`${id}-date`} type="date" required value={eatenOn} max={today}
                            onChange={e => setEatenOn(e.target.value)} className={PUBLIC_INPUT} />
                    </Field>
                    <Field label={t("meal")} htmlFor={`${id}-meal`}>
                        <select id={`${id}-meal`} value={meal} onChange={e => setMeal(e.target.value as Meal)} className={PUBLIC_INPUT}>
                            {MEALS.map(m => <option key={m} value={m}>{t(`meals.${m}`)}</option>)}
                        </select>
                    </Field>

                    <Field label={t("store")} htmlFor={`${id}-store`}>
                        <input id={`${id}-store`} value={store} maxLength={MAX_STORE} list={`${id}-stores`}
                            onChange={e => setStore(e.target.value)} className={PUBLIC_INPUT} />
                        <datalist id={`${id}-stores`}>
                            {stores.map(s => <option key={s} value={s} />)}
                        </datalist>
                    </Field>
                    <Field label={t("item")} htmlFor={`${id}-item`}>
                        <input id={`${id}-item`} value={item} required maxLength={MAX_ITEM} list={`${id}-items`}
                            autoFocus={!isEdit}
                            onChange={e => setItem(e.target.value)} className={PUBLIC_INPUT} />
                        <datalist id={`${id}-items`}>
                            {itemOptions.map(s => <option key={s} value={s} />)}
                        </datalist>
                    </Field>

                    <Field label={t("amount")} htmlFor={`${id}-amount`} hint={t("amountHint")}>
                        <input id={`${id}-amount`} type="number" inputMode="numeric" min={0} max={MAX_AMOUNT} step={1}
                            value={amount} onChange={e => setAmount(e.target.value)} className={PUBLIC_INPUT} />
                    </Field>
                    <Field label={t("quantity")} htmlFor={`${id}-qty`}>
                        <input id={`${id}-qty`} type="number" inputMode="numeric" required min={1} max={MAX_QUANTITY} step={1}
                            value={quantity} onChange={e => setQuantity(e.target.value)} className={PUBLIC_INPUT} />
                    </Field>
                </div>

                <Field label={t("rating")}>
                    <RatingInput value={rating} onChange={setRating} />
                </Field>

                <Field label={t("note")} htmlFor={`${id}-note`}>
                    <textarea id={`${id}-note`} value={note} rows={3} maxLength={MAX_NOTE}
                        onChange={e => setNote(e.target.value)} className={cn(PUBLIC_INPUT, "w-full")} />
                </Field>

                <Field label={t("mealLabel")} htmlFor={`${id}-label`} hint={t("mealLabelHint")}>
                    <input id={`${id}-label`} value={mealLabel} maxLength={MAX_MEAL_LABEL}
                        onChange={e => setMealLabel(e.target.value)} className={PUBLIC_INPUT} />
                </Field>

                <div className="flex items-center gap-2">
                    {onDelete && (
                        <button
                            type="button"
                            onClick={() => run(onDelete)}
                            disabled={busy}
                            className="flex items-center gap-1 px-3 py-2 text-sm rounded-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 disabled:opacity-50"
                        >
                            <Trash2 size={16} />
                            {t("delete")}
                        </button>
                    )}
                    <div className="ml-auto flex gap-2">
                        <button
                            type="button"
                            onClick={onClose}
                            disabled={busy}
                            className="px-4 py-2 text-sm rounded-sm border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700 disabled:opacity-50"
                        >
                            {t("cancel")}
                        </button>
                        <button
                            type="submit"
                            disabled={busy}
                            className="px-4 py-2 text-sm rounded-sm bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50"
                        >
                            {busy ? "…" : t("save")}
                        </button>
                    </div>
                </div>
            </form>
        </Modal>
    );
}

function Field({ label, htmlFor, hint, children }: {
    label: string;
    htmlFor?: string;
    hint?: string;
    children: React.ReactNode;
}) {
    return (
        <div className="flex flex-col gap-1 min-w-0">
            {htmlFor ? <label htmlFor={htmlFor} className={LABEL}>{label}</label> : <span className={LABEL}>{label}</span>}
            {children}
            {hint && <span className="text-xs text-neutral-500 dark:text-neutral-400">{hint}</span>}
        </div>
    );
}

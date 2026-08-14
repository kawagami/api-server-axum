"use client";

import { useCallback } from "react";
import { useTranslations } from "next-intl";

/**
 * 記帳分類 value → 當前語系標籤。
 *
 * `GET /member/ledger/categories` 回的 `label` 是後端寫死的繁中，直接渲染等於 en / zh-CN
 * 使用者看到中文。分類清單是後端固定常數（`structs/ledger.rs` 的 EXPENSE_CATEGORIES /
 * INCOME_CATEGORIES），故 value 當 i18n key、後端 label 只當未知 key 的 fallback
 * ——後端加了新分類而三語系還沒補時，畫面退回中文而不是空白。
 */
export default function useLedgerCategoryLabel() {
    const t = useTranslations("LedgerCategories");
    return useCallback(
        (value: string, fallback?: string) => (t.has(value) ? t(value) : (fallback ?? value)),
        [t]
    );
}

/**
 * 滑鼠移到欄位上就聚焦並全選（鬧鐘／倒數原本各自的 UX）。
 * 三張卡片併到同一頁後多了「滑過另一張卡把焦點搶走」的風險，
 * 所以已經有別的輸入框在編輯時就不搶。
 */
export function selectOnHover(el: HTMLInputElement | null, disabled: boolean) {
    if (disabled || !el) return;
    const active = document.activeElement;
    if (active instanceof HTMLInputElement && active !== el) return;
    el.focus();
    el.select();
}

export const NUMBER_INPUT =
    "w-full p-3 border border-neutral-300 dark:border-neutral-600 rounded-lg shadow-xs bg-white dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100";

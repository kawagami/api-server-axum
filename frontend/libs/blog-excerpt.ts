// 從 blog markdown 產生純文字摘要（列表卡片預覽用）。
// 純函式、零 DOM，故 server component 可直接呼叫；同時可測。

/**
 * 把 markdown 粗略轉純文字：移除標題符號、粗斜體、連結/圖片、程式碼區塊、清單符號等，
 * 供列表卡片顯示前幾行預覽。不追求完美渲染，只求可讀掃視。
 */
export function markdownToPlainText(markdown: string): string {
    return markdown
        // 移除 fenced code block（含內容）
        .replace(/```[\s\S]*?```/g, ' ')
        // 圖片 ![alt](url) → 去除
        .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')
        // 連結 [text](url) → 只留 text
        .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
        // 行首標題 / 引用 / 清單符號
        .replace(/^\s{0,3}(#{1,6}|>|[-*+]|\d+\.)\s+/gm, '')
        // 粗體 / 斜體 / 行內碼 的標記符號
        .replace(/[*_`~]+/g, '')
        // 表格分隔線
        .replace(/^\s*\|?[\s:|-]+\|?\s*$/gm, ' ')
        // 連續空白（含換行）壓成單一空格
        .replace(/\s+/g, ' ')
        .trim();
}

/**
 * 產生指定長度上限的摘要，超過截斷並補「…」。
 * 會先跳過與標題重複的開頭（列表卡片已單獨顯示標題）。
 */
export function makeExcerpt(markdown: string, title: string, maxLen = 120): string {
    let text = markdownToPlainText(markdown);
    const plainTitle = markdownToPlainText(title);
    // 開頭若就是標題，去掉避免與卡片標題重複
    if (plainTitle && text.startsWith(plainTitle)) {
        text = text.slice(plainTitle.length).trim();
    }
    if (text.length <= maxLen) return text;
    return text.slice(0, maxLen).trimEnd() + '…';
}

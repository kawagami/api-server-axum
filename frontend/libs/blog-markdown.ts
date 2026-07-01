import GithubSlugger from 'github-slugger';

export interface Heading {
    level: number;
    text: string;
    slug: string;
}

// 把 markdown 粗略轉純文字（去掉語法符號），給 <meta description> / og:description 用
export function markdownToPlainText(markdown: string): string {
    return markdown
        .replace(/```[\s\S]*?```/g, ' ')            // fenced code block
        .replace(/`[^`]*`/g, ' ')                    // inline code
        .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')       // images
        .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')     // links → 文字
        .replace(/^#{1,6}\s+/gm, '')                 // headings
        .replace(/^\s*>+\s?/gm, '')                  // blockquotes
        .replace(/^\s*[-*+]\s+/gm, '')               // list bullets
        .replace(/[*_~]/g, '')                       // emphasis marks
        .replace(/\s+/g, ' ')
        .trim();
}

// 取前 maxLen 字當摘要，截斷時補「…」
export function extractExcerpt(markdown: string, maxLen = 155): string {
    const text = markdownToPlainText(markdown);
    if (text.length <= maxLen) return text;
    return text.slice(0, maxLen).trimEnd() + '…';
}

// 抽出標題清單；slug 用 github-slugger，與 rehype-slug 產生的 heading id 一致（含重複去重）
export function extractHeadings(markdown: string): Heading[] {
    const slugger = new GithubSlugger();
    const withoutCode = markdown.replace(/```[\s\S]*?```/g, '');   // 避免 code block 內的 # 被當標題
    const matches = withoutCode.match(/^#{1,6}[ \t]+.+$/gm) ?? [];
    return matches.map((line) => {
        const level = line.match(/^#+/)![0].length;
        const text = line.replace(/^#{1,6}[ \t]+/, '').replace(/\s+#*\s*$/, '').trim();
        return { level, text, slug: slugger.slug(text) };
    });
}

// markdown 內第一張圖片 URL，給 og:image fallback
export function firstImageUrl(markdown: string): string | null {
    return markdown.match(/!\[[^\]]*\]\(([^)\s]+)/)?.[1] ?? null;
}

import { cache } from "react";
import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import type { Metadata } from "next";
import { getBlog } from "@/api/blogs";
import { apiErrorStatus } from "@/libs/api-error";
import BlogArticle from "@/components/blogs/blog-article";
import CommentSection from "@/components/blogs/comments/comment-section";
import { extractExcerpt, firstImageUrl } from "@/libs/blog-markdown";
import { localeAlternates } from "@/libs/seo";

/**
 * 找不到的文章回 `null` 而不是丟例外 —— 網址列的 id 是使用者可以亂打的，而後端對
 * 非 UUID 的 id 回 **400**（不是 404）。不接的話 `fetchApi` 會把它丟成未處理例外，
 * 訪客拿到的是 500 錯誤頁而不是 404 頁（`notFound()` 在這層仍是 soft 404 ——
 * HTTP 200 + 404 內容，見 frontend CLAUDE.md，但至少不是錯誤頁）。
 * 其餘狀態（後端掛掉的 5xx、逾時）照樣丟，那是真的該亮紅燈。
 */
const fetchBlog = cache(async (id: string) => {
    try {
        return await getBlog(id);
    } catch (e) {
        const status = apiErrorStatus(e);
        if (status === 400 || status === 404) return null;
        throw e;
    }
});

type Params = Promise<{ id: string; locale: string }>;

export async function generateMetadata({ params }: { params: Params }): Promise<Metadata> {
    const { id, locale } = await params;
    const blog = await fetchBlog(id);
    // 404 的 metadata 不重要，但 generateMetadata 先於頁面執行，這裡不擋就會 crash
    if (!blog) return { title: "Not Found | Kawa's Blog" };
    const title = blog.tocs[0] ?? "Blog";
    const description = extractExcerpt(blog.markdown);
    const image = firstImageUrl(blog.markdown);
    const url = `/${locale}/blogs/${id}`;

    return {
        title: `${title} | Kawa's Blog`,
        description,
        alternates: localeAlternates(locale, `/blogs/${id}`),
        openGraph: {
            type: "article",
            title,
            description,
            url,
            images: image ? [image] : undefined,
            publishedTime: blog.created_at,
            modifiedTime: blog.updated_at,
        },
        twitter: {
            card: "summary_large_image",
            title,
            description,
            images: image ? [image] : undefined,
        },
    };
}

export default async function BlogPage({ params }: { params: Params }) {
    const { id } = await params;
    const blog = await fetchBlog(id);
    if (!blog) notFound();
    // 訪客(無 access_token)也能留言,登入會員留言則綁身分;此處僅判斷是否顯示訪客名欄
    const isMember = !!(await cookies()).get("access_token")?.value;

    const jsonLd = {
        "@context": "https://schema.org",
        "@type": "BlogPosting",
        headline: blog.tocs[0] ?? "Blog",
        description: extractExcerpt(blog.markdown),
        datePublished: blog.created_at,
        dateModified: blog.updated_at,
        keywords: blog.tags?.join(", "),
        author: { "@type": "Person", name: "Kawa" },
    };

    return (
        <>
            {/*
                `<` 必須 escape：JSON.stringify 不會處理它，而 headline / description /
                keywords 三者都來自 PUT /admin/blogs/:id 的 body（tocs 是原封存下來的，
                markdown 經 markdownToPlainText 也不去角括號）。作者塞一個 `</script>`
                就能跳出這個元素、在公開文章頁執行任意 script。
                JSON 字串裡 `<` 與 `<` 等價，schema.org 解析不受影響。
            */}
            <script
                type="application/ld+json"
                dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd).replace(/</g, "\\u003c") }}
            />
            <BlogArticle
                markdown={blog.markdown}
                comments={<CommentSection blogId={id} isMember={isMember} />}
            />
        </>
    );
}

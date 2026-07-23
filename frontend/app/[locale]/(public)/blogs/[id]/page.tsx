import { cache } from "react";
import { cookies } from "next/headers";
import type { Metadata } from "next";
import { getBlog } from "@/api/blogs";
import BlogArticle from "@/components/blogs/blog-article";
import CommentSection from "@/components/blogs/comments/comment-section";
import { extractExcerpt, firstImageUrl } from "@/libs/blog-markdown";

const fetchBlog = cache(getBlog);

type Params = Promise<{ id: string; locale: string }>;

export async function generateMetadata({ params }: { params: Params }): Promise<Metadata> {
    const { id, locale } = await params;
    const blog = await fetchBlog(id);
    const title = blog.tocs[0] ?? "Blog";
    const description = extractExcerpt(blog.markdown);
    const image = firstImageUrl(blog.markdown);
    const url = `/${locale}/blogs/${id}`;

    return {
        title: `${title} | Kawa's Blog`,
        description,
        alternates: { canonical: url },
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
            <script
                type="application/ld+json"
                dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
            />
            <BlogArticle
                markdown={blog.markdown}
                comments={<CommentSection blogId={id} isMember={isMember} />}
            />
        </>
    );
}

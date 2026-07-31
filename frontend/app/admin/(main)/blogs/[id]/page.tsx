import { getBlog } from "@/api/blogs";
import { getBlogTags } from "@/api/blogs";
import { getPublicSettings } from "@/api/settings";
import BlogComponent from "@/components/blogs/blog-component";
import PageHeader from "@/components/admin/page-header";
import { resolveImageCompressConfig } from "@/libs/image-config";
import type { Blog } from "@/types";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "編輯文章",
    description: "Markdown 編輯與預覽",
};

export default async function BlogPage({ params }: { params: Promise<{ id: string }> }) {
    const id = (await params).id;
    const [blogResult, allTags, publicSettings] = await Promise.all([
        getBlog(id).catch((e: Error): Blog => {
            if (e.message.includes('API 404')) return { id, markdown: '', tags: [], tocs: [] };
            throw e;
        }),
        getBlogTags(),
        getPublicSettings(),
    ]);

    // 標題不帶 description：編輯器是滿版固定高（blog-component 的 100svh-224px），
    // 多一行說明就要再改那個公式與 loading.tsx，得不償失
    return (
        <div className="flex flex-col gap-4">
            <PageHeader title="編輯文章" />
            <BlogComponent id={id} blog={blogResult} allTags={allTags} compressConfig={resolveImageCompressConfig(publicSettings)} />
        </div>
    );
}

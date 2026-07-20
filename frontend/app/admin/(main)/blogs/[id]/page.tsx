import { getBlog } from "@/api/blogs";
import { getBlogTags } from "@/api/blogs";
import { getPublicSettings } from "@/api/settings";
import BlogComponent from "@/components/blogs/blog-component";
import { resolveImageCompressConfig } from "@/libs/image-config";
import type { Blog } from "@/types";

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

    return <BlogComponent id={id} blog={blogResult} allTags={allTags} compressConfig={resolveImageCompressConfig(publicSettings)} />;
}

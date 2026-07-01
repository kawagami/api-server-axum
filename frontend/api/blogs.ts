"use server";

import { revalidateTag } from "next/cache";
import { fetchApi } from "@/libs/fetchApi";
import adminRequest from "@/libs/adminRequest";
import type { Blog, BlogInput, BlogPaginatedResponse } from "@/types";

interface GetBlogsParams {
    page?: number;
    per_page?: number;
    tag?: string | null;
}

// blog 內容近乎靜態：用 Next Data Cache + tag 失效取代 no-store
// （layout 讀 cookies() 強制動態渲染，故只能靠 fetch data cache，無法 SSG）
// 寫入時 putBlog / deleteBlog 會 revalidateTag('blogs')，故快取期間不會看到舊資料
export async function getBlogs({ page = 1, per_page = 10, tag }: GetBlogsParams = {}): Promise<BlogPaginatedResponse> {
    const params = new URLSearchParams({ page: String(page), per_page: String(per_page) });
    if (tag) params.set('tag', tag);
    return fetchApi(`${process.env.API_URL}/blogs?${params}`, { next: { revalidate: 60, tags: ['blogs'] } });
}

export async function getBlog(id: string): Promise<Blog> {
    return fetchApi(`${process.env.API_URL}/blogs/${id}`, { next: { revalidate: 300, tags: ['blogs', `blog:${id}`] } });
}

export async function getBlogTags(): Promise<string[]> {
    return fetchApi(`${process.env.API_URL}/blogs/tags`, { next: { tags: ['blogs'] } });
}

export async function putBlog(id: string, blog: BlogInput): Promise<void> {
    await adminRequest({
        url: `${process.env.API_URL}/admin/blogs/${id}`,
        headers: { 'Content-Type': 'application/json' },
        method: 'PUT',
        body: JSON.stringify(blog),
    });
    revalidateTag('blogs', 'max');
    revalidateTag(`blog:${id}`, 'max');
}

export async function deleteBlog(id: string): Promise<void> {
    await adminRequest({
        url: `${process.env.API_URL}/admin/blogs/${id}`,
        method: 'DELETE',
    });
    revalidateTag('blogs', 'max');
    revalidateTag(`blog:${id}`, 'max');
}

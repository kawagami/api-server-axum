"use server";

import { revalidateTag } from "next/cache";
import { fetchApi } from "@/libs/fetchApi";
import adminRequest from "@/libs/adminRequest";
import type { Blog, BlogInput, BlogPaginatedResponse, TagCount } from "@/types";

interface GetBlogsParams {
    page?: number;
    per_page?: number;
    tag?: string | null;
    author?: string | null;
    /** 關鍵字搜尋（比對文章內容） */
    q?: string | null;
    /** 排序：oldest = 舊到新；其餘 = 新到舊 */
    sort?: string | null;
}

// blog 內容近乎靜態：用 Next Data Cache + tag 失效取代 no-store
// （layout 讀 cookies() 強制動態渲染，故只能靠 fetch data cache，無法 SSG）
// 寫入時 putBlog / deleteBlog 會 revalidateTag('blogs')，故快取期間不會看到舊資料
export async function getBlogs({ page = 1, per_page = 10, tag, author, q, sort }: GetBlogsParams = {}): Promise<BlogPaginatedResponse> {
    const params = new URLSearchParams({ page: String(page), per_page: String(per_page) });
    if (tag) params.set('tag', tag);
    if (author) params.set('author', author);
    if (q) params.set('q', q);
    if (sort) params.set('sort', sort);
    return fetchApi(`${process.env.API_URL}/blogs?${params}`, { next: { revalidate: 60, tags: ['blogs'] } });
}

// 後台管理列表：一般 admin 只拿自己的文章、super_admin 全拿（走 adminRequest 認證，不快取跨使用者）
export async function getAdminBlogs({ page = 1, per_page = 200 }: { page?: number; per_page?: number } = {}): Promise<BlogPaginatedResponse> {
    const params = new URLSearchParams({ page: String(page), per_page: String(per_page) });
    return adminRequest<BlogPaginatedResponse>({ url: `${process.env.API_URL}/admin/blogs?${params}` });
}

export async function getBlog(id: string): Promise<Blog> {
    return fetchApi(`${process.env.API_URL}/blogs/${id}`, { next: { revalidate: 300, tags: ['blogs', `blog:${id}`] } });
}

export async function getBlogTags(): Promise<string[]> {
    return fetchApi(`${process.env.API_URL}/blogs/tags`, { next: { tags: ['blogs'] } });
}

// 公開列表側欄用：每個 tag 附文章數
export async function getBlogTagCounts(): Promise<TagCount[]> {
    return fetchApi(`${process.env.API_URL}/blogs/tags/counts`, { next: { tags: ['blogs'] } });
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

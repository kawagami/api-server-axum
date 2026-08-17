"use server";

import { updateTag } from "next/cache";
import { fetchApi } from "@/libs/fetchApi";
import adminRequest from "@/libs/adminRequest";
import type { AdminBlogPaginatedResponse, Blog, BlogInput, BlogPaginatedResponse, TagCount } from "@/types";

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
// 寫入時 putBlog / deleteBlog 會 updateTag('blogs')，故快取期間不會看到舊資料
// ⚠️ 寫入路徑一律用 updateTag，不用 revalidateTag(tag, 'max')：
//    revalidateTag 帶了非 expire:0 的 profile 時，Next 刻意不設 pathWasRevalidated
//    （避免 server action 讀到自己的寫入），連帶**不會失效瀏覽器的 Router Cache**，
//    結果存檔後再點編輯會拿到 client 端快取的舊 RSC payload，非 hard reload 不會更新。
export async function getBlogs({ page = 1, per_page = 10, tag, author, q, sort }: GetBlogsParams = {}): Promise<BlogPaginatedResponse> {
    const params = new URLSearchParams({ page: String(page), per_page: String(per_page) });
    if (tag) params.set('tag', tag);
    if (author) params.set('author', author);
    if (q) params.set('q', q);
    if (sort) params.set('sort', sort);
    return fetchApi(`${process.env.API_URL}/blogs?${params}`, { next: { revalidate: 60, tags: ['blogs'] } });
}

interface GetAdminBlogsParams {
    page?: number;
    per_page?: number;
    tag?: string | null;
    /** 關鍵字搜尋（比對文章內容） */
    q?: string | null;
    /** 排序：oldest = 建立時間舊到新；updated = 更新時間新到舊；其餘 = 建立時間新到舊 */
    sort?: string | null;
}

// 後台管理列表：一般 admin 只拿自己的文章、super_admin 全拿（走 adminRequest 認證，不快取跨使用者）。
// 回的列不含 markdown（見 AdminBlogListItem）；擁有者由 session 決定，不吃 author 參數。
export async function getAdminBlogs({ page = 1, per_page = 50, tag, q, sort }: GetAdminBlogsParams = {}): Promise<AdminBlogPaginatedResponse> {
    const params = new URLSearchParams({ page: String(page), per_page: String(per_page) });
    if (tag) params.set('tag', tag);
    if (q) params.set('q', q);
    if (sort) params.set('sort', sort);
    const res = await adminRequest<AdminBlogPaginatedResponse>({ url: `${process.env.API_URL}/admin/blogs?${params}` });
    return res ?? { data: [], total: 0 };
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
    updateTag('blogs');
    updateTag(`blog:${id}`);
}

export async function deleteBlog(id: string): Promise<void> {
    await adminRequest({
        url: `${process.env.API_URL}/admin/blogs/${id}`,
        method: 'DELETE',
    });
    updateTag('blogs');
    updateTag(`blog:${id}`);
}

// 全站改名/合併 tag（一般 admin 只影響自己的文章，super_admin 全站）。回受影響文章數。
export async function renameBlogTag(from: string, to: string): Promise<number> {
    const res = await adminRequest<{ affected: number }>({
        url: `${process.env.API_URL}/admin/blogs/tags`,
        headers: { 'Content-Type': 'application/json' },
        method: 'PATCH',
        body: JSON.stringify({ from, to }),
    });
    updateTag('blogs');
    return res?.affected ?? 0;
}

// 全站移除某 tag。回受影響文章數。
export async function deleteBlogTag(tag: string): Promise<number> {
    const res = await adminRequest<{ affected: number }>({
        url: `${process.env.API_URL}/admin/blogs/tags?tag=${encodeURIComponent(tag)}`,
        method: 'DELETE',
    });
    updateTag('blogs');
    return res?.affected ?? 0;
}

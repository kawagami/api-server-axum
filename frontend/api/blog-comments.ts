"use server";

import { cookies } from "next/headers";
import { fetchApi } from "@/libs/fetchApi";
import adminRequest from "@/libs/adminRequest";
import type { BlogComment, NewBlogComment, BlogCommentListResponse } from "@/types";

// 公開端:單篇 blog 留言列表(舊到新,不需登入)
export async function getBlogComments(
    blogId: string,
    page = 1,
    per_page = 50,
): Promise<BlogComment[]> {
    const res = await fetchApi<BlogCommentListResponse>(
        `${process.env.API_URL}/blogs/${blogId}/comments?page=${page}&per_page=${per_page}`,
        { cache: "no-store" },
    );
    return res.data;
}

// 公開端:提交留言。有 access_token cookie 就帶上(後端 optional-auth 綁 member_id),否則為訪客
export async function postBlogComment(blogId: string, input: NewBlogComment): Promise<BlogComment> {
    const token = (await cookies()).get("access_token")?.value;
    return fetchApi<BlogComment>(`${process.env.API_URL}/blogs/${blogId}/comments`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            ...(token && { Authorization: `Bearer ${token}` }),
        },
        body: JSON.stringify(input),
        cache: "no-store",
    });
}

interface AdminCommentListResponse {
    data: BlogComment[];
    total: number;
}

// 後台:全站留言分頁列表(需 comment:read)
export async function getAllBlogComments(page = 1, per_page = 50): Promise<BlogComment[]> {
    const res = await adminRequest<AdminCommentListResponse>({
        url: `${process.env.API_URL}/admin/blog_comments?page=${page}&per_page=${per_page}`,
    });
    return res?.data ?? [];
}

// 後台:刪除留言(需 comment:delete)
export async function deleteBlogComment(id: number): Promise<void> {
    await adminRequest<null>({
        url: `${process.env.API_URL}/admin/blog_comments/${id}`,
        method: "DELETE",
    });
}

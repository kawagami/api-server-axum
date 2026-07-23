// blog 留言(對應後端 backend/src/structs/blog_comments.rs 的 BlogComment / NewComment)
// is_member=true 為會員留言(author_name/avatar_url 取自 members 表);false 為訪客(author_name 為自填名,可能為 null)
export interface BlogComment {
    id: number;
    blog_id: string;
    content: string;
    created_at: string;
    is_member: boolean;
    author_name: string | null;
    avatar_url: string | null;
}

export interface NewBlogComment {
    content: string;
    name?: string;
}

export interface BlogCommentListResponse {
    data: BlogComment[];
    total: number;
}

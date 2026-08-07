import type { PaginatedResponse } from './pagination';

// Blog
export interface Blog {
  id: string;
  markdown: string;
  tags: string[];
  tocs: string[];
  created_at?: string;
  updated_at?: string;
  author_name?: string | null;
}

export interface TagCount {
  tag: string;
  count: number;
}

// 後端 2026-08-03 起回 `Paginated<T>`（`{ data, total }`）；原本另有 page / per_page，
// 但那兩欄只是把 request 參數回抄給 client，全站無人讀，已隨後端一起移除
export type BlogPaginatedResponse = PaginatedResponse<Blog>;

// PUT /admin/blogs/:id 的 body。**沒有 tocs** ——
// 標題與目錄由後端從 markdown 解析（services/blogs.rs 的 extract_toc_texts），
// 原本由編輯器自己 parse 一份送上去，等於讓 client 決定文章標題
export interface BlogInput {
  markdown: string;
  tags: string[];
}

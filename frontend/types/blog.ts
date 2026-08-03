import type { PaginatedResponse } from './pagination';

// Blog
export interface Toc {
  id: string;
  level: number;
  text: string;
}

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

export interface BlogInput {
  markdown: string;
  tags: string[];
  tocs: Toc[];
}

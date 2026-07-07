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

export interface BlogPaginatedResponse {
  total: number;
  page: number;
  per_page: number;
  data: Blog[];
}

export interface BlogInput {
  markdown: string;
  tags: string[];
  tocs: Toc[];
}

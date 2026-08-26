use crate::{
    errors::{AppError, RequestError},
    repositories::{blogs as blogs_repo, images as images_repo},
    structs::auth::AuthenticatedUser,
    structs::blogs::{AdminBlogFilter, AdminBlogListItem, AdminBlogSort, DbBlog, PutBlog, TagCount},
    structs::pagination::{PageQuery, Paginated}
};
use regex::Regex;
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::sync::OnceLock;
use uuid::Uuid;

static MD_IMAGE_RE: OnceLock<Regex> = OnceLock::new();

/// 寫入前正規化 tag 陣列：去頭尾空白、去空字串、大小寫不敏感去重（保留首次出現的顯示形）。
/// 避免同篇出現「Rust / rust」或含空白的重複標籤。跨篇的統一改由後台 rename/merge 處理。
pub fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_lowercase()) {
            out.push(trimmed.to_string());
        }
    }
    out
}

/// 抓出 markdown 內文所有圖片 URL（`![](url)`），不篩來源。
/// 是否為站內上傳圖交給下游 `images` table 的 `url = ANY($1)` 精確比對決定：
/// 外部圖（如 https://example.com/x.png）比不中任何 row、無害。
/// 刻意不以 host/path 字串（如 `/uploads/`）判斷，公開 URL 網域才能與儲存後端解耦
/// （media.kawa.homes / 未來 R2 / 商家 instance 皆適用）。
fn extract_image_urls(markdown: &str) -> Vec<String> {
    let re = MD_IMAGE_RE
        .get_or_init(|| Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").expect("static regex is always valid"));

    re.captures_iter(markdown)
        .map(|cap| cap[1].to_string())
        .collect()
}

static MD_FENCE_RE: OnceLock<Regex> = OnceLock::new();
static MD_HEADING_RE: OnceLock<Regex> = OnceLock::new();
static MD_HEADING_TRAILER_RE: OnceLock<Regex> = OnceLock::new();

/// 從 markdown 抽出各級標題文字（`tocs` 欄位、`tocs[0]` 即文章標題）。
///
/// 這件事原本是**前端**做的：編輯器 `blog-component.tsx` 自己 parse 一份 `tocs` 隨
/// `PUT /admin/blogs/:id` 送上來，後端照單全收。但後端手上就有完整 markdown，
/// 沒有理由信任 client 算的衍生資料 —— 送不匹配的 tocs 就能讓標題與內文對不上
/// （標題會進列表卡片、WS 廣播與 SEO title）。改由這裡產生後，
/// 請求 body 只剩 `{ markdown, tags }`，也不必再要求 client 產出它用不到的 `{id, level}`。
///
/// 規則對齊前端顯示端的 `libs/blog-markdown.ts::extractHeadings`（TOC 側欄用的那份）：
/// 先剝掉 fenced code block（否則程式碼裡的 `# 註解` 會被當標題），
/// 再去掉行尾的 closing `#`。
pub fn extract_toc_texts(markdown: &str) -> Vec<String> {
    let fence = MD_FENCE_RE
        .get_or_init(|| Regex::new(r"(?s)```.*?```").expect("static regex is always valid"));
    let heading = MD_HEADING_RE
        .get_or_init(|| Regex::new(r"(?m)^#{1,6}[ \t]+(.+)$").expect("static regex is always valid"));
    let trailer = MD_HEADING_TRAILER_RE
        .get_or_init(|| Regex::new(r"\s+#*\s*$").expect("static regex is always valid"));

    let without_code = fence.replace_all(markdown, "");
    heading
        .captures_iter(&without_code)
        .map(|cap| trailer.replace(cap[1].trim_end(), "").trim().to_string())
        .filter(|text| !text.is_empty())
        .collect()
}

/// 搜尋關鍵字的長度上限（**字元數**，不是 bytes —— 中文一字 3 bytes，用 len() 會誤殺）。
///
/// ⚠️ **這個上限不是效能防線,別把它當成防線。** 加它的時候假設「ILIKE 成本 ∝ 關鍵字長度」,
/// 2026-08-27 實測(2000 篇 / 3MB 表)推翻了那個假設：4000 字的 pattern（1.3ms）並沒有比
/// 100 字（2.4ms）貴 —— 長 pattern 在每個位置都能提早拒絕,反而更快。
///
/// 真正貴的是**命中大量文章的常見詞**：`%後端%` 要 27ms,而列表端點的 `count` 那一半
/// 無法靠 LIMIT 提早結束,一個請求 count + list 併發兩份 ≈ 60ms 的 PG CPU。
/// 那件事由兩層擋,都不在這裡：
/// - `deploy/nginx/nginx.conf` 的 `search` zone（帶 `?q=` 才計數，20r/m）—— 主要防線
/// - `blogs.markdown` 的 pg_trgm GIN 索引（migration 20260827000000）—— 讓選擇性高的
///   關鍵字從 2.3ms 降到 0.13ms，但對「全中」的詞無效
///
/// 留這個上限的理由退回到最樸素的那個：**別讓無界的使用者輸入一路走進 SQL 與 log**。
/// 100 字對真實搜尋綽綽有餘（實際輸入多為 2–10 字），成本是零。
pub const MAX_SEARCH_LEN: usize = 100;

/// 關鍵字長度檢查。公開與後台兩條列表共用 —— 後台雖需認證，但成本模型相同，沒有理由放寬。
fn validate_search(q: Option<&str>) -> Result<(), AppError> {
    match q {
        Some(q) if q.chars().count() > MAX_SEARCH_LEN => Err(RequestError::UnprocessableContent(
            format!("搜尋關鍵字上限為 {MAX_SEARCH_LEN} 字"),
        )
        .into()),
        _ => Ok(()),
    }
}

pub async fn get_blogs(
    pool: &Pool<Postgres>,
    page: &PageQuery,
    tag: Option<String>,
    author: Option<String>,
    q: Option<String>,
    sort: Option<String>,
) -> Result<Paginated<DbBlog>, AppError> {
    let (per_page, offset) = page.to_limit_offset(10);
    let tag_ref = tag.as_deref();
    let author_ref = author.as_deref();
    // 關鍵字空白視同無過濾；排序只認 oldest，其餘一律 newest
    let q_ref = q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    validate_search(q_ref)?;
    let ascending = sort.as_deref() == Some("oldest");
    let (total, data) = tokio::try_join!(
        blogs_repo::count_blogs(pool, tag_ref, author_ref, q_ref),
        blogs_repo::get_blogs_with_pagination(pool, per_page, offset, tag_ref, author_ref, q_ref, ascending),
    )?;
    Ok(Paginated::new(data, total))
}

pub async fn get_blog(pool: &Pool<Postgres>, id: Uuid) -> Result<DbBlog, AppError> {
    blogs_repo::get_blog_by_id(pool, id).await
}

/// 後台管理列表（依擁有者過濾；super_admin 傳 None 看全部）。公開列表仍走 get_blogs。
pub async fn get_admin_blogs(
    pool: &Pool<Postgres>,
    owner_id: Option<i64>,
    page: &PageQuery,
    filter: AdminBlogFilter,
) -> Result<Paginated<AdminBlogListItem>, AppError> {
    let (per_page, offset) = page.to_limit_offset(50);
    // 空白字串視同無過濾（前端的空欄位會照送），排序走白名單列舉
    let tag = filter.tag.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let q = filter.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    validate_search(q)?;
    let sort = AdminBlogSort::from_query(filter.sort.as_deref());
    let (total, data) = tokio::try_join!(
        blogs_repo::count_for_owner(pool, owner_id, tag, q),
        blogs_repo::list_for_owner(pool, owner_id, tag, q, sort, per_page, offset),
    )?;
    Ok(Paginated::new(data, total))
}

pub async fn get_tags(pool: &Pool<Postgres>) -> Result<Vec<String>, AppError> {
    blogs_repo::get_all_tags(pool).await
}

pub async fn get_tag_counts(pool: &Pool<Postgres>) -> Result<Vec<TagCount>, AppError> {
    blogs_repo::get_tag_counts(pool).await
}

/// 全站改名/合併 tag。owner=None → super_admin 動全部；Some(id) → 只動自己的文章。
pub async fn rename_tag(
    pool: &Pool<Postgres>,
    owner: Option<i64>,
    from: String,
    to: String,
) -> Result<u64, AppError> {
    let from = from.trim();
    let to = to.trim();
    if from.is_empty() || to.is_empty() {
        return Err(RequestError::UnprocessableContent("tag 不可為空".into()).into());
    }
    if from == to {
        return Ok(0);
    }
    blogs_repo::rename_tag(pool, owner, from, to).await
}

/// 全站移除某 tag。owner 語意同 `rename_tag`。
pub async fn delete_tag(pool: &Pool<Postgres>, owner: Option<i64>, tag: String) -> Result<u64, AppError> {
    let tag = tag.trim();
    if tag.is_empty() {
        return Err(RequestError::UnprocessableContent("tag 不可為空".into()).into());
    }
    blogs_repo::delete_tag(pool, owner, tag).await
}

/// 建立或更新一篇文章，回傳標題。
///
/// **擁有者檢查在這裡**：既有文章只能改自己的（super_admin 例外），
/// 不存在＝新建、擁有者記為 actor。放在 service 是因為「誰能改這篇」跟寫入是同一個
/// 決策，留在 route 就得靠每個呼叫端自己記得先查一次 author。
pub async fn upsert_blog(
    pool: &Pool<Postgres>,
    actor: &AuthenticatedUser,
    id: Uuid,
    blog: PutBlog,
) -> Result<String, AppError> {
    if let Some(author) = blogs_repo::get_author(pool, id).await? {
        actor.require_owner(author)?;
    }
    let author_id = actor.id;
    upsert_blog_inner(pool, id, blog, author_id).await
}

async fn upsert_blog_inner(pool: &Pool<Postgres>, id: Uuid, blog: PutBlog, author_id: i64) -> Result<String, AppError> {
    let tocs = extract_toc_texts(&blog.markdown);
    let title = tocs.first().cloned().unwrap_or_default();

    let old_urls = match blogs_repo::get_blog_by_id(pool, id).await {
        Ok(old_blog) => extract_image_urls(&old_blog.markdown),
        Err(AppError::RequestError(RequestError::NotFound)) => vec![],
        Err(e) => return Err(e),
    };

    let new_urls = extract_image_urls(&blog.markdown);
    let new_url_set: HashSet<&String> = new_urls.iter().collect();
    let orphaned_urls: Vec<String> = old_urls.into_iter().filter(|u| !new_url_set.contains(u)).collect();

    let orphaned_ids: Vec<i32> = if orphaned_urls.is_empty() {
        vec![]
    } else {
        images_repo::get_images_by_urls(pool, &orphaned_urls)
            .await?
            .into_iter()
            .map(|r| r.id)
            .collect()
    };

    let tags = normalize_tags(blog.tags);

    let mut tx = pool.begin().await?;
    blogs_repo::upsert_blog_in_tx(&mut tx, id, blog.markdown, tocs, tags, author_id).await?;
    if !new_urls.is_empty() {
        images_repo::mark_images_active_by_urls_in_tx(&mut tx, &new_urls).await?;
    }
    if !orphaned_ids.is_empty() {
        images_repo::mark_images_unused_by_ids_in_tx(&mut tx, &orphaned_ids).await?;
    }
    tx.commit().await?;

    Ok(title)
}

/// 刪文（連同其圖片）。擁有者檢查同 `upsert_blog`；查不到作者＝文章不存在 → 404。
pub async fn delete_blog_with_images(
    pool: &Pool<Postgres>,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<(), AppError> {
    let author = blogs_repo::get_author(pool, id)
        .await?
        .ok_or(RequestError::NotFound)?;
    actor.require_owner(author)?;
    delete_blog_with_images_inner(pool, id).await
}

async fn delete_blog_with_images_inner(pool: &Pool<Postgres>, id: Uuid) -> Result<(), AppError> {
    let blog = blogs_repo::get_blog_by_id(pool, id).await?;
    let upload_urls = extract_image_urls(&blog.markdown);

    let image_ids: Vec<i32> = if upload_urls.is_empty() {
        vec![]
    } else {
        images_repo::get_images_by_urls(pool, &upload_urls)
            .await?
            .into_iter()
            .map(|r| r.id)
            .collect()
    };

    let mut tx = pool.begin().await?;
    blogs_repo::delete_blog_in_tx(&mut tx, id).await?;
    if !image_ids.is_empty() {
        images_repo::mark_images_unused_by_ids_in_tx(&mut tx, &image_ids).await?;
    }
    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{extract_toc_texts, normalize_tags, validate_search, MAX_SEARCH_LEN};

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn trims_and_drops_empty() {
        assert_eq!(normalize_tags(v(&["  rust ", "", "   ", "後端"])), v(&["rust", "後端"]));
    }

    #[test]
    fn dedupes_case_insensitively_keeping_first_display_form() {
        assert_eq!(normalize_tags(v(&["Rust", "rust", "RUST"])), v(&["Rust"]));
    }

    #[test]
    fn dedupes_after_trim() {
        assert_eq!(normalize_tags(v(&["tag", " tag "])), v(&["tag"]));
    }

    #[test]
    fn preserves_order() {
        assert_eq!(normalize_tags(v(&["b", "a", "c", "A"])), v(&["b", "a", "c"]));
    }

    #[test]
    fn extracts_headings_in_document_order() {
        let md = "# 標題\n\n內文\n\n## 小節 A\n\n### 更小的\n";
        assert_eq!(extract_toc_texts(md), v(&["標題", "小節 A", "更小的"]));
    }

    /// tocs[0] 就是文章標題，會進列表卡片 / WS 廣播 / SEO title。
    /// code block 裡的 `#` 註解若被當成標題，整篇文章的標題就變成一行程式碼註解
    #[test]
    fn ignores_hashes_inside_fenced_code_blocks() {
        let md = "# 真標題\n\n```bash\n# 這是註解不是標題\necho hi\n```\n\n## 真小節\n";
        assert_eq!(extract_toc_texts(md), v(&["真標題", "真小節"]));
    }

    #[test]
    fn strips_closing_hashes_and_trailing_space() {
        assert_eq!(extract_toc_texts("## 置中式標題 ##\n"), v(&["置中式標題"]));
        assert_eq!(extract_toc_texts("# 有尾巴空白   \n"), v(&["有尾巴空白"]));
    }

    /// `#hashtag`（無空白）與 7 個以上的 `#` 都不是 markdown 標題
    #[test]
    fn requires_space_and_at_most_six_hashes() {
        assert!(extract_toc_texts("#沒空白\n").is_empty());
        assert!(extract_toc_texts("####### 七個井字\n").is_empty());
    }

    #[test]
    fn no_headings_yields_empty_title() {
        assert!(extract_toc_texts("只有內文，沒有任何標題").is_empty());
    }

    #[test]
    fn search_accepts_none_and_short_keywords() {
        assert!(validate_search(None).is_ok());
        assert!(validate_search(Some("rust")).is_ok());
        assert!(validate_search(Some("繁體中文關鍵字")).is_ok());
    }

    /// 界線本身：剛好 100 字放行、101 字擋下
    #[test]
    fn search_length_is_capped_at_the_boundary() {
        let ok = "字".repeat(MAX_SEARCH_LEN);
        let too_long = "字".repeat(MAX_SEARCH_LEN + 1);
        assert!(validate_search(Some(&ok)).is_ok());
        assert!(validate_search(Some(&too_long)).is_err());
    }

    /// 上限算的是字元不是 bytes —— 用 `len()` 的話 34 個中文字（102 bytes）就會被誤殺
    #[test]
    fn search_length_counts_chars_not_bytes() {
        let cjk = "字".repeat(MAX_SEARCH_LEN);
        assert!(cjk.len() > MAX_SEARCH_LEN, "前提：中文一字多於 1 byte");
        assert!(validate_search(Some(&cjk)).is_ok());
    }
}

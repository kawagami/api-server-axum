use crate::{
    errors::{AppError, RequestError},
    repositories::{blogs as blogs_repo, images as images_repo},
    structs::blogs::{BlogsResponse, DbBlog, PutBlog, TagCount},
    structs::pagination::PageQuery,
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

pub async fn get_blogs(
    pool: &Pool<Postgres>,
    page: &PageQuery,
    tag: Option<String>,
    author: Option<String>,
    q: Option<String>,
    sort: Option<String>,
) -> Result<BlogsResponse, AppError> {
    let (per_page, offset) = page.to_limit_offset(10);
    let (page, per_page, offset) = (page.page.unwrap_or(1).max(1) as usize, per_page as usize, offset as usize);
    let tag_ref = tag.as_deref();
    let author_ref = author.as_deref();
    // 關鍵字空白視同無過濾；排序只認 oldest，其餘一律 newest
    let q_ref = q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let ascending = sort.as_deref() == Some("oldest");
    let (total, data) = tokio::try_join!(
        blogs_repo::count_blogs(pool, tag_ref, author_ref, q_ref),
        blogs_repo::get_blogs_with_pagination(pool, per_page, offset, tag_ref, author_ref, q_ref, ascending),
    )?;
    Ok(BlogsResponse { total, page, per_page, data })
}

pub async fn get_blog(pool: &Pool<Postgres>, id: Uuid) -> Result<DbBlog, AppError> {
    blogs_repo::get_blog_by_id(pool, id).await
}

/// 後台管理列表（依擁有者過濾；super_admin 傳 None 看全部）。公開列表仍走 get_blogs。
pub async fn get_admin_blogs(
    pool: &Pool<Postgres>,
    owner_id: Option<i64>,
    page: &PageQuery,
) -> Result<BlogsResponse, AppError> {
    let (per_page, offset) = page.to_limit_offset(50);
    let (page_no, per_page_usize) = (page.page.unwrap_or(1).max(1) as usize, per_page as usize);
    let (total, data) = tokio::try_join!(
        blogs_repo::count_for_owner(pool, owner_id),
        blogs_repo::list_for_owner(pool, owner_id, per_page, offset),
    )?;
    Ok(BlogsResponse { total, page: page_no, per_page: per_page_usize, data })
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

pub async fn upsert_blog(pool: &Pool<Postgres>, id: Uuid, blog: PutBlog, author_id: i64) -> Result<String, AppError> {
    let tocs = blog.extract_toc_texts();
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

pub async fn delete_blog_with_images(pool: &Pool<Postgres>, id: Uuid) -> Result<(), AppError> {
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
    use super::normalize_tags;

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
}

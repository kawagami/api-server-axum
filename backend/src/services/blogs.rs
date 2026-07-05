use crate::{
    errors::{AppError, RequestError},
    repositories::{blogs as blogs_repo, images as images_repo},
    structs::blogs::{BlogsResponse, DbBlog, PutBlog},
    structs::pagination::PageQuery,
};
use regex::Regex;
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::sync::OnceLock;
use uuid::Uuid;

static MD_IMAGE_RE: OnceLock<Regex> = OnceLock::new();

fn extract_upload_urls(markdown: &str) -> Vec<String> {
    let re = MD_IMAGE_RE
        .get_or_init(|| Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").expect("static regex is always valid"));

    re.captures_iter(markdown)
        .filter_map(|cap| {
            let url = cap[1].to_string();
            if url.contains("/uploads/") { Some(url) } else { None }
        })
        .collect()
}

pub async fn get_blogs(
    pool: &Pool<Postgres>,
    page: &PageQuery,
    tag: Option<String>,
) -> Result<BlogsResponse, AppError> {
    let (per_page, offset) = page.to_limit_offset(10);
    let (page, per_page, offset) = (page.page.unwrap_or(1).max(1) as usize, per_page as usize, offset as usize);
    let tag_ref = tag.as_deref();
    let (total, data) = tokio::try_join!(
        blogs_repo::count_blogs(pool, tag_ref),
        blogs_repo::get_blogs_with_pagination(pool, per_page, offset, tag_ref),
    )?;
    Ok(BlogsResponse { total, page, per_page, data })
}

pub async fn get_blog(pool: &Pool<Postgres>, id: Uuid) -> Result<DbBlog, AppError> {
    blogs_repo::get_blog_by_id(pool, id).await
}

pub async fn get_tags(pool: &Pool<Postgres>) -> Result<Vec<String>, AppError> {
    blogs_repo::get_all_tags(pool).await
}

pub async fn upsert_blog(pool: &Pool<Postgres>, id: Uuid, blog: PutBlog) -> Result<String, AppError> {
    let tocs = blog.extract_toc_texts();
    let title = tocs.first().cloned().unwrap_or_default();

    let old_urls = match blogs_repo::get_blog_by_id(pool, id).await {
        Ok(old_blog) => extract_upload_urls(&old_blog.markdown),
        Err(AppError::RequestError(RequestError::NotFound)) => vec![],
        Err(e) => return Err(e),
    };

    let new_urls = extract_upload_urls(&blog.markdown);
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

    let mut tx = pool.begin().await?;
    blogs_repo::upsert_blog_in_tx(&mut tx, id, blog.markdown, tocs, blog.tags).await?;
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
    let upload_urls = extract_upload_urls(&blog.markdown);

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

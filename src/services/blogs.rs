use crate::{
    errors::AppError,
    repositories::{blogs as blogs_repo, images as images_repo},
    state::AppStateV2,
    structs::blogs::{DbBlog, PutBlog},
};
use regex::Regex;
use std::sync::OnceLock;
use uuid::Uuid;

static MD_IMAGE_RE: OnceLock<Regex> = OnceLock::new();

fn extract_upload_urls(markdown: &str) -> Vec<String> {
    let re = MD_IMAGE_RE
        .get_or_init(|| Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").unwrap());

    re.captures_iter(markdown)
        .filter_map(|cap| {
            let url = cap[1].to_string();
            if url.contains("/uploads/") { Some(url) } else { None }
        })
        .collect()
}

pub async fn get_blogs(
    state: &AppStateV2,
    page: usize,
    per_page: usize,
) -> Result<Vec<DbBlog>, AppError> {
    let offset = (page.saturating_sub(1)) * per_page;
    blogs_repo::get_blogs_with_pagination(state, per_page, offset).await
}

pub async fn get_blog(state: &AppStateV2, id: Uuid) -> Result<DbBlog, AppError> {
    blogs_repo::get_blog_by_id(state, id).await
}

pub async fn upsert_blog(state: &AppStateV2, id: Uuid, blog: PutBlog) -> Result<(), AppError> {
    let tocs = blog.extract_toc_texts();
    blogs_repo::upsert_blog(state, id, blog.markdown, tocs, blog.tags).await
}

pub async fn delete_blog_with_images(state: &AppStateV2, id: Uuid) -> Result<(), AppError> {
    let blog = blogs_repo::get_blog_by_id(state, id).await?;
    let upload_urls = extract_upload_urls(&blog.markdown);

    let images = if upload_urls.is_empty() {
        vec![]
    } else {
        images_repo::get_images_by_urls(state, &upload_urls).await?
    };

    let image_ids: Vec<i32> = images.iter().map(|r| r.id).collect();
    let storage_keys: Vec<String> = images.iter().map(|r| r.storage_key.clone()).collect();

    let mut tx = state.get_pool().begin().await?;

    blogs_repo::delete_blog_in_tx(&mut tx, id).await?;

    if !image_ids.is_empty() {
        images_repo::delete_images_in_tx(&mut tx, &image_ids).await?;
    }

    tx.commit().await?;

    for key in &storage_keys {
        if let Err(e) = state.get_storage().delete(key).await {
            tracing::error!("failed to delete file {}: {}", key, e);
        }
    }

    Ok(())
}

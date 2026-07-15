use crate::{
    errors::{AppError, RequestError, SystemError},
    repositories::images as images_repo,
    repositories::images::ImageRecord,
    storage::Storage,
};
use axum::{body::Bytes, extract::Multipart};
use image::{DynamicImage, ImageFormat, ImageReader, Limits};
use sqlx::{Pool, Postgres};
use std::io::Cursor;

/// decode 階段的資源上限，擋 decode-bomb（小檔解壓成超大點陣）。
/// admin-only 上傳、低併發，1核1G 下 256MB 足夠涵蓋正常大圖又能擋惡意炸彈。
const MAX_DECODE_ALLOC: u64 = 256 * 1024 * 1024;

/// 驗證 bytes 確實是可解碼的圖片，並統一重編碼為 lossless WebP。
///
/// - 用 `image` crate 實際 decode 一次 = 最強的「真的是圖片」驗證（不是只看副檔名/magic bytes）。
/// - 非圖片或損毀 → `InvalidContent`（400），不會存進磁碟。
/// - lossless WebP 編碼器僅收 RGB8 / RGBA8，故先依有無 alpha 正規化色彩型別。
/// - 重編碼順帶剝除 EXIF 等 metadata。
fn process_to_webp(data: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| RequestError::InvalidContent(format!("無法辨識圖片格式: {e}")))?;

    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    reader.limits(limits);

    let img = reader
        .decode()
        .map_err(|e| RequestError::InvalidContent(format!("不是有效的圖片: {e}")))?;

    let normalized = if img.color().has_alpha() {
        DynamicImage::ImageRgba8(img.into_rgba8())
    } else {
        DynamicImage::ImageRgb8(img.into_rgb8())
    };

    let mut out = Vec::new();
    normalized
        .write_to(&mut Cursor::new(&mut out), ImageFormat::WebP)
        .map_err(|e| SystemError::Internal(format!("WebP 編碼失敗: {e}")))?;
    Ok(out)
}

pub async fn get_images(pool: &Pool<Postgres>, owner_id: Option<i64>) -> Result<Vec<ImageRecord>, AppError> {
    images_repo::get_all_images(pool, owner_id).await
}

pub async fn cleanup_unused_images(pool: &Pool<Postgres>, storage: &Storage) {
    let records = match images_repo::take_old_unused_images(pool).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("cleanup_unused_images db error: {}", e);
            return;
        }
    };
    for r in &records {
        if let Err(e) = storage.delete(&r.storage_key).await {
            tracing::error!("cleanup_unused_images storage delete failed {}: {}", r.storage_key, e);
        }
    }
}

pub async fn delete_image(pool: &Pool<Postgres>, storage: &Storage, id: i32) -> Result<(), AppError> {
    let storage_key = images_repo::delete_image_by_id(pool, id).await?;
    if let Err(e) = storage.delete(&storage_key).await {
        tracing::error!("storage delete failed for key {}: {}", storage_key, e);
    }
    Ok(())
}

pub async fn upload_images(
    pool: &Pool<Postgres>,
    storage: &Storage,
    base_url: &str,
    owner_id: Option<i64>,
    mut multipart: Multipart,
) -> Result<Vec<ImageRecord>, AppError> {
    let mut records = vec![];

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| RequestError::MultipartError(e.into()))?
    {
        // 先整份讀進記憶體（受全域 RequestBodyLimit 10MB 保護），才能 decode 驗證 + 轉檔。
        let data = field
            .bytes()
            .await
            .map_err(|e| RequestError::MultipartError(e.into()))?;

        // 驗證是圖片並轉成 WebP；非圖片會在這裡以 400 擋掉，不落磁碟。
        let webp = process_to_webp(&data)?;

        // storage 仍走 stream 介面，把處理後的 bytes 包成單元素 stream。
        let body = futures::stream::once(async move { Ok::<Bytes, std::io::Error>(Bytes::from(webp)) });
        let (storage_key, url) = storage
            .upload(body, "image/webp", base_url)
            .await
            .map_err(|e| RequestError::MultipartError(e.into()))?;
        let record = images_repo::insert_image(pool, &storage_key, &url, owner_id).await?;
        records.push(record);
    }

    if records.is_empty() {
        return Err(RequestError::InvalidContent("no file provided".into()).into());
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_png(w: u32, h: u32, alpha: bool) -> Vec<u8> {
        let img = if alpha {
            DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(w, h, image::Rgba([200, 50, 50, 128])))
        } else {
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(w, h, image::Rgb([200, 50, 50])))
        };
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png).unwrap();
        buf
    }

    #[test]
    fn valid_png_becomes_webp() {
        for alpha in [false, true] {
            let webp = process_to_webp(&make_png(4, 4, alpha)).unwrap();
            // RIFF....WEBP 檔頭確認確實編成 WebP
            assert_eq!(&webp[0..4], b"RIFF", "alpha={alpha}");
            assert_eq!(&webp[8..12], b"WEBP", "alpha={alpha}");
            // 且輸出可被重新解碼回圖片
            assert!(image::load_from_memory(&webp).is_ok(), "alpha={alpha}");
        }
    }

    #[test]
    fn non_image_bytes_are_rejected() {
        let err = process_to_webp(b"this is definitely not an image").unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }

    #[test]
    fn truncated_image_is_rejected() {
        let mut png = make_png(8, 8, false);
        png.truncate(png.len() / 2); // 砍半 = 損毀
        let err = process_to_webp(&png).unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }
}

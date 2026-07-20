use crate::{
    errors::{AppError, RequestError, SystemError},
    repositories::images as images_repo,
    repositories::images::ImageRecord,
    storage::Storage,
};
use axum::extract::Multipart;
use image::{ImageFormat, ImageReader, Limits};
use sqlx::{Pool, Postgres};
use std::io::Cursor;

/// decode 階段的資源上限，擋 decode-bomb（小檔解壓成超大點陣）。
const MAX_DECODE_ALLOC: u64 = 128 * 1024 * 1024;
/// 單邊尺寸上限 = libwebp 編碼上限（16383），超過的圖在 decode 前就以 400 擋掉。
const MAX_DIMENSION: u32 = 16383;
/// 總像素上限（40MP ≈ RGBA 160MB），約束 decode 後色彩正規化的第二份緩衝，避免 1G RAM 上 OOM。
const MAX_PIXELS: u64 = 40_000_000;
/// lossy WebP 品質（0-100）。
const WEBP_QUALITY: f32 = 80.0;

#[derive(Debug)]
pub struct ProcessedImage {
    pub bytes: Vec<u8>,
    pub ext: &'static str,
}

/// 驗證 bytes 確實是可解碼的圖片，並重編碼為 lossy WebP（q80）。
///
/// - 用 `image` crate 實際 decode 一次 = 最強的「真的是圖片」驗證（不是只看副檔名/magic bytes）。
/// - 非圖片、損毀、尺寸/像素超限 → `InvalidContent`（400），不會存進磁碟。
/// - GIF 例外：decode 驗證後保留原檔（重編碼只會取第一幀，動畫會被毀掉）。
/// - libwebp encoder 僅收 RGB8 / RGBA8，故先依有無 alpha 正規化色彩型別。
/// - 重編碼順帶剝除 EXIF 等 metadata。
///
/// CPU 密集（decode + encode 可達秒級），caller 必須包在 `spawn_blocking` 執行。
pub fn process_image(data: &[u8]) -> Result<ProcessedImage, AppError> {
    let mut reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| RequestError::InvalidContent(format!("無法辨識圖片格式: {e}")))?;
    let format = reader.format();

    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    reader.limits(limits);

    let img = reader
        .decode()
        .map_err(|e| RequestError::InvalidContent(format!("不是有效的圖片: {e}")))?;

    // GIF 可能是動畫，重編碼會只剩第一幀；decode 已驗證合法，原檔直接保留
    if format == Some(ImageFormat::Gif) {
        return Ok(ProcessedImage { bytes: data.to_vec(), ext: "gif" });
    }

    let (w, h) = (img.width(), img.height());
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(RequestError::InvalidContent(format!("圖片像素過大: {w}x{h}")).into());
    }

    let webp = if img.color().has_alpha() {
        webp::Encoder::from_rgba(&img.into_rgba8(), w, h).encode_simple(false, WEBP_QUALITY)
    } else {
        webp::Encoder::from_rgb(&img.into_rgb8(), w, h).encode_simple(false, WEBP_QUALITY)
    }
    .map_err(|e| SystemError::Internal(format!("WebP 編碼失敗: {e:?}")))?;

    Ok(ProcessedImage { bytes: webp.to_vec(), ext: "webp" })
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

/// 單檔上傳：取 multipart 第一個檔案欄位，驗證+轉檔+落地+寫 DB，回傳該筆記錄。
///
/// 前端一律「一張一請求」（client 端壓縮 pipeline），故不再迴圈收多檔：
/// 較小的 request body 避開 Cloudflare HTTP/3 大 POST 卡死，且每檔獨立成功/失敗，
/// 不會有「批次中一檔壞掉、其餘已落地變孤兒」的髒狀態。
pub async fn upload_image(
    pool: &Pool<Postgres>,
    storage: &Storage,
    base_url: &str,
    owner_id: Option<i64>,
    mut multipart: Multipart,
) -> Result<ImageRecord, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| RequestError::MultipartError(e.into()))?
    {
        // 只處理檔案欄位，跳過表單文字欄位
        if field.file_name().is_none() {
            continue;
        }

        // 先整份讀進記憶體（受全域 RequestBodyLimit 10MB 保護），才能 decode 驗證 + 轉檔
        let data = field
            .bytes()
            .await
            .map_err(|e| RequestError::MultipartError(e.into()))?;

        // CPU 密集的 decode + encode 走 spawn_blocking，不卡住 tokio worker（1 核機上會凍結全站）
        let processed = tokio::task::spawn_blocking(move || process_image(&data))
            .await
            .map_err(|e| SystemError::Internal(format!("圖片處理任務失敗: {e}")))??;

        // 寫檔失敗是伺服器故障（磁碟滿/權限），回 500 而非 4xx
        let (storage_key, url) = storage
            .upload(&processed.bytes, processed.ext, base_url)
            .await
            .map_err(|e| SystemError::Internal(format!("儲存圖片失敗: {e}")))?;
        return images_repo::insert_image(pool, &storage_key, &url, owner_id).await;
    }

    Err(RequestError::InvalidContent("no file provided".into()).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

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
            let processed = process_image(&make_png(4, 4, alpha)).unwrap();
            assert_eq!(processed.ext, "webp", "alpha={alpha}");
            // RIFF....WEBP 檔頭確認確實編成 WebP
            assert_eq!(&processed.bytes[0..4], b"RIFF", "alpha={alpha}");
            assert_eq!(&processed.bytes[8..12], b"WEBP", "alpha={alpha}");
            // 且輸出可被重新解碼回圖片
            assert!(image::load_from_memory(&processed.bytes).is_ok(), "alpha={alpha}");
        }
    }

    #[test]
    fn gif_is_kept_as_original() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 4, image::Rgb([10, 20, 30])));
        let mut gif = Vec::new();
        img.write_to(&mut Cursor::new(&mut gif), ImageFormat::Gif).unwrap();

        let processed = process_image(&gif).unwrap();
        assert_eq!(processed.ext, "gif");
        assert_eq!(processed.bytes, gif); // 原檔 byte-for-byte 保留（動畫不被壓平）
    }

    #[test]
    fn non_image_bytes_are_rejected() {
        let err = process_image(b"this is definitely not an image").unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }

    #[test]
    fn truncated_image_is_rejected() {
        let mut png = make_png(8, 8, false);
        png.truncate(png.len() / 2); // 砍半 = 損毀
        let err = process_image(&png).unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }

    #[test]
    fn oversized_dimension_is_rejected() {
        // 寬 > 16383（libwebp 上限），decode 階段就該以 400 擋下,而不是編碼時 500
        let err = process_image(&make_png(MAX_DIMENSION + 1, 1, false)).unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }
}

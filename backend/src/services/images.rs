use crate::{
    errors::{AppError, RequestError, SystemError},
    repositories::images::{self as images_repo, NewImage},
    structs::{auth::AuthenticatedUser, images::{ImagePlaceholder, ImageRecord}},
    storage::Storage,
};
use axum::extract::Multipart;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use image::{DynamicImage, ImageFormat, ImageReader, Limits};
use sqlx::{Pool, Postgres};
use std::io::Cursor;

/// decode 階段的資源上限，擋 decode-bomb（小檔解壓成超大點陣）。
const MAX_DECODE_ALLOC: u64 = 128 * 1024 * 1024;
/// 單邊尺寸上限 = libwebp 編碼上限（16383），超過的圖在 decode 前就以 400 擋掉。
const MAX_DIMENSION: u32 = 16383;
/// 總像素上限（40MP ≈ RGBA 160MB），約束 decode 後色彩正規化的第二份緩衝，避免 1G RAM 上 OOM。
const MAX_PIXELS: u64 = 40_000_000;
/// lossy WebP 預設品質（0-100）；設定缺失/無法解析時的 fallback。
pub const DEFAULT_WEBP_QUALITY: f32 = 80.0;
/// 模糊預覽小圖的長邊 px。next/image 會自己放大 + 高斯模糊，官方建議 ≤10px 級；
/// 16px 的 WebP 約 100–200 bytes，base64 後隨 blog 內文 JSON 一起下發仍可忽略。
const BLUR_MAX_EDGE: u32 = 16;
/// 模糊預覽小圖的 WebP 品質；反正要被模糊，壓低省 bytes。
const BLUR_WEBP_QUALITY: f32 = 50.0;

#[derive(Debug)]
pub struct ProcessedImage {
    pub bytes: Vec<u8>,
    pub ext: &'static str,
    pub placeholder: ImagePlaceholder,
}

/// 帶資源上限的 decode；非圖片、損毀、單邊超限 → `InvalidContent`（400）。
fn decode_limited(data: &[u8]) -> Result<(DynamicImage, Option<ImageFormat>), AppError> {
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
    Ok((img, format))
}

/// 原圖寬高 + 長邊 `BLUR_MAX_EDGE` 的 WebP data URL（GIF 取第一幀）。
/// 小圖編碼失敗不擋上傳：`blur_data_url` 為 None，前端退回無 placeholder。
fn make_placeholder(img: &DynamicImage) -> ImagePlaceholder {
    let thumb = img.thumbnail(BLUR_MAX_EDGE, BLUR_MAX_EDGE);
    let (w, h) = (thumb.width(), thumb.height());
    let encoded = if thumb.color().has_alpha() {
        webp::Encoder::from_rgba(&thumb.to_rgba8(), w, h).encode_simple(false, BLUR_WEBP_QUALITY)
    } else {
        webp::Encoder::from_rgb(&thumb.to_rgb8(), w, h).encode_simple(false, BLUR_WEBP_QUALITY)
    };
    let blur_data_url = match encoded {
        Ok(webp) => Some(format!("data:image/webp;base64,{}", BASE64.encode(&*webp))),
        Err(e) => {
            tracing::warn!("模糊預覽 WebP 編碼失敗: {e:?}");
            None
        }
    };
    // 單邊已被 MAX_DIMENSION（16383）擋住，轉 i32 不會溢位
    ImagePlaceholder { width: img.width() as i32, height: img.height() as i32, blur_data_url }
}

/// 驗證 bytes 確實是可解碼的圖片，並重編碼為 lossy WebP。
///
/// - 用 `image` crate 實際 decode 一次 = 最強的「真的是圖片」驗證（不是只看副檔名/magic bytes）。
/// - 非圖片、損毀、尺寸/像素超限 → `InvalidContent`（400），不會存進磁碟。
/// - GIF 例外：decode 驗證後保留原檔（重編碼只會取第一幀，動畫會被毀掉）。
/// - libwebp encoder 僅收 RGB8 / RGBA8，故先依有無 alpha 正規化色彩型別。
/// - 重編碼順帶剝除 EXIF 等 metadata。
/// - `quality`（0–100）為 WebP 品質，由 caller 從 `app_settings.image_webp_quality` 傳入。
/// - 順手產生模糊預覽（`placeholder`），圖已 decode 在手，成本只剩一次縮圖。
///
/// CPU 密集（decode + encode 可達秒級），caller 必須包在 `spawn_blocking` 執行。
pub fn process_image(data: &[u8], quality: f32) -> Result<ProcessedImage, AppError> {
    let (img, format) = decode_limited(data)?;

    // GIF 可能是動畫，重編碼會只剩第一幀；decode 已驗證合法，原檔直接保留
    if format == Some(ImageFormat::Gif) {
        return Ok(ProcessedImage { bytes: data.to_vec(), ext: "gif", placeholder: make_placeholder(&img) });
    }

    let (w, h) = (img.width(), img.height());
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(RequestError::InvalidContent(format!("圖片像素過大: {w}x{h}")).into());
    }

    let placeholder = make_placeholder(&img);
    let quality = quality.clamp(1.0, 100.0);
    let webp = if img.color().has_alpha() {
        webp::Encoder::from_rgba(&img.into_rgba8(), w, h).encode_simple(false, quality)
    } else {
        webp::Encoder::from_rgb(&img.into_rgb8(), w, h).encode_simple(false, quality)
    }
    .map_err(|e| SystemError::Internal(format!("WebP 編碼失敗: {e:?}")))?;

    Ok(ProcessedImage { bytes: webp.to_vec(), ext: "webp", placeholder })
}

/// 舊圖回填寬高 + 模糊預覽（欄位是 2026-09-30 才加的）。啟動時由 `routes.rs` spawn 跑一次。
///
/// 一次只 decode 一張（`spawn_blocking` 依序等待），1 核 1G 上不會同時堆多份點陣。
/// 讀檔 / decode 失敗的列維持 NULL（前端退回無 placeholder），最後彙總一筆 WARN；
/// 下次啟動會再試一次，所以常駐的 WARN = DB 有列但檔案不見或損毀，值得看一眼。
pub async fn backfill_placeholders(pool: &Pool<Postgres>, storage: &Storage) {
    const BATCH: i64 = 20;
    let (mut after_id, mut filled, mut failed) = (0, 0usize, Vec::new());
    loop {
        let rows = match images_repo::list_missing_placeholders(pool, after_id, BATCH).await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!("backfill_placeholders 查詢失敗: {e}");
                return;
            }
        };
        let Some(&(last_id, _)) = rows.last() else { break };
        after_id = last_id;

        for (id, storage_key) in rows {
            let result = async {
                let data = storage
                    .read(&storage_key)
                    .await
                    .map_err(|e| SystemError::Internal(format!("讀檔失敗: {e}")))?;
                let (img, _) = tokio::task::spawn_blocking(move || decode_limited(&data))
                    .await
                    .map_err(|e| SystemError::Internal(format!("decode 任務失敗: {e}")))??;
                let p = make_placeholder(&img);
                images_repo::set_placeholder(pool, id, p.width, p.height, p.blur_data_url.as_deref()).await
            }
            .await;
            match result {
                Ok(()) => filled += 1,
                Err(e) => failed.push(format!("#{id} {storage_key}: {e}")),
            }
        }
    }
    if filled > 0 {
        tracing::info!("backfill_placeholders: 補齊 {filled} 張");
    }
    if !failed.is_empty() {
        tracing::warn!("backfill_placeholders: {} 張失敗（維持無預覽）: {}", failed.len(), failed.join("; "));
    }
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

/// 刪圖。**擁有者檢查在這裡**（不是 route）：`require_owner` 對非擁有者回 404，
/// 那是業務規則的一部分，跟「刪 DB 再刪檔」是同一個決策；留在 route 會讓每個
/// 呼叫端各自記得先查一次 owner，漏掉沒有任何徵兆。
pub async fn delete_image(
    pool: &Pool<Postgres>,
    storage: &Storage,
    actor: &AuthenticatedUser,
    id: i32,
) -> Result<(), AppError> {
    actor.require_owner(images_repo::get_owner(pool, id).await?)?;
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
    quality: f32,
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
        let processed = tokio::task::spawn_blocking(move || process_image(&data, quality))
            .await
            .map_err(|e| SystemError::Internal(format!("圖片處理任務失敗: {e}")))??;

        // 寫檔失敗是伺服器故障（磁碟滿/權限），回 500 而非 4xx
        let (storage_key, url) = storage
            .upload(&processed.bytes, processed.ext, base_url)
            .await
            .map_err(|e| SystemError::Internal(format!("儲存圖片失敗: {e}")))?;
        let p = &processed.placeholder;
        return images_repo::insert_image(
            pool,
            NewImage {
                storage_key: &storage_key,
                url: &url,
                owner_id,
                width: p.width,
                height: p.height,
                blur_data_url: p.blur_data_url.as_deref(),
            },
        )
        .await;
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
            let processed = process_image(&make_png(4, 4, alpha), DEFAULT_WEBP_QUALITY).unwrap();
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

        let processed = process_image(&gif, DEFAULT_WEBP_QUALITY).unwrap();
        assert_eq!(processed.ext, "gif");
        assert_eq!(processed.bytes, gif); // 原檔 byte-for-byte 保留（動畫不被壓平）
    }

    /// 解開 data URL，回傳小圖的 (寬, 高)
    fn blur_dimensions(p: &ImagePlaceholder) -> (u32, u32) {
        let url = p.blur_data_url.as_deref().expect("應有模糊預覽");
        let b64 = url.strip_prefix("data:image/webp;base64,").expect("data URL 前綴");
        let img = image::load_from_memory(&BASE64.decode(b64).unwrap()).unwrap();
        (img.width(), img.height())
    }

    #[test]
    fn placeholder_keeps_original_size_and_aspect() {
        for alpha in [false, true] {
            let processed = process_image(&make_png(400, 100, alpha), DEFAULT_WEBP_QUALITY).unwrap();
            let p = &processed.placeholder;
            assert_eq!((p.width, p.height), (400, 100), "alpha={alpha}");
            // 長邊縮到 BLUR_MAX_EDGE，比例維持 4:1
            assert_eq!(blur_dimensions(p), (BLUR_MAX_EDGE, BLUR_MAX_EDGE / 4), "alpha={alpha}");
        }
    }

    #[test]
    fn gif_gets_placeholder_from_first_frame() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(30, 60, image::Rgb([10, 20, 30])));
        let mut gif = Vec::new();
        img.write_to(&mut Cursor::new(&mut gif), ImageFormat::Gif).unwrap();

        let p = process_image(&gif, DEFAULT_WEBP_QUALITY).unwrap().placeholder;
        assert_eq!((p.width, p.height), (30, 60));
        assert_eq!(blur_dimensions(&p), (BLUR_MAX_EDGE / 2, BLUR_MAX_EDGE));
    }

    #[test]
    fn non_image_bytes_are_rejected() {
        let err = process_image(b"this is definitely not an image", DEFAULT_WEBP_QUALITY).unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }

    #[test]
    fn truncated_image_is_rejected() {
        let mut png = make_png(8, 8, false);
        png.truncate(png.len() / 2); // 砍半 = 損毀
        let err = process_image(&png, DEFAULT_WEBP_QUALITY).unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }

    #[test]
    fn oversized_dimension_is_rejected() {
        // 寬 > 16383（libwebp 上限），decode 階段就該以 400 擋下,而不是編碼時 500
        let err = process_image(&make_png(MAX_DIMENSION + 1, 1, false), DEFAULT_WEBP_QUALITY).unwrap_err();
        assert!(matches!(err, AppError::RequestError(RequestError::InvalidContent(_))));
    }
}

use image::ImageFormat;
use std::io::{Cursor, Error, ErrorKind, Result};

/// 將字串映射為 ImageFormat
fn str_to_image_format(format: &str) -> Result<ImageFormat> {
    match format.to_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::WebP),
        "bmp" => Ok(ImageFormat::Bmp),
        "gif" => Ok(ImageFormat::Gif),
        "ico" => Ok(ImageFormat::Ico),
        "tiff" => Ok(ImageFormat::Tiff),
        _ => Ok(ImageFormat::Png),
    }
}

pub fn resize_image(data: &[u8], width: u32, height: u32, output_format: &str) -> Result<Vec<u8>> {
    // 解析輸出的格式
    let image_format = str_to_image_format(output_format)?;

    // 載入圖片
    let img = image::load_from_memory(data)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Image loading failed: {:?}", e)))?;

    // 調整大小
    let resized_img = img.resize(width, height, image::imageops::FilterType::Triangle);

    // 設定緩衝區
    let mut buffer = Cursor::new(Vec::new());

    // 將調整大小後的圖片寫入目標格式
    resized_img
        .write_to(&mut buffer, image_format)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Image writing failed: {:?}", e)))?;

    Ok(buffer.into_inner())
}

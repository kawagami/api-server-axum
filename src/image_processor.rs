use image::ImageFormat;
use std::io::{Cursor, Error, ErrorKind, Result};

pub fn resize_image(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    // 將 ImageError 顯式映射為 std::io::Error
    let img = image::load_from_memory(data)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Image loading failed: {:?}", e)))?;

    let resized_img = img.resize(width, height, image::imageops::FilterType::Triangle);

    let mut buffer = Cursor::new(Vec::new());
    resized_img
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Image writing failed: {:?}", e)))?;

    Ok(buffer.into_inner())
}

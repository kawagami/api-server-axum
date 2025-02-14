use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    #[serde(default = "default_count")]
    pub count: u8,
    #[serde(default = "default_length")]
    pub length: u8,
}

fn default_count() -> u8 {
    1
}

fn default_length() -> u8 {
    8
}

// 圖片格式的型別定義
#[derive(Debug, Clone)]
pub enum ImageFormat {
    PNG,
    WEBP,
    JPEG,
    BMP,
    GIF,
    ICO,
    TIFF,
}

impl ImageFormat {
    pub fn content_type(&self) -> &'static str {
        match self {
            ImageFormat::PNG => "image/png",
            ImageFormat::WEBP => "image/webp",
            ImageFormat::JPEG => "image/jpeg",
            ImageFormat::BMP => "image/bmp",
            ImageFormat::GIF => "image/gif",
            ImageFormat::ICO => "image/x-icon",
            ImageFormat::TIFF => "image/tiff",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "png" => Some(Self::PNG),
            "webp" => Some(Self::WEBP),
            "jpeg" | "jpg" => Some(Self::JPEG),
            "bmp" => Some(Self::BMP),
            "gif" => Some(Self::GIF),
            "ico" => Some(Self::ICO),
            "tiff" => Some(Self::TIFF),
            _ => None,
        }
    }
}

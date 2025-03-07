use serde::{Deserialize, Serialize};
use std::str::FromStr;

// 常數定義
pub const DEFAULT_PASSWORD_COUNT: u8 = 1;
pub const DEFAULT_PASSWORD_LENGTH: u8 = 8;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Params {
    #[serde(default = "default_count")]
    pub count: u8,
    #[serde(default = "default_length")]
    pub length: u8,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            count: DEFAULT_PASSWORD_COUNT,
            length: DEFAULT_PASSWORD_LENGTH,
        }
    }
}

fn default_count() -> u8 {
    DEFAULT_PASSWORD_COUNT
}

fn default_length() -> u8 {
    DEFAULT_PASSWORD_LENGTH
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ImageFormat {
    Png,
    Webp,
    Jpeg,
    Bmp,
    Gif,
    Ico,
    Tiff,
}

impl ImageFormat {
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Webp => "image/webp",
            Self::Jpeg => "image/jpeg",
            Self::Bmp => "image/bmp",
            Self::Gif => "image/gif",
            Self::Ico => "image/x-icon",
            Self::Tiff => "image/tiff",
        }
    }
}

impl FromStr for ImageFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "webp" => Ok(Self::Webp),
            "jpeg" | "jpg" => Ok(Self::Jpeg),
            "bmp" => Ok(Self::Bmp),
            "gif" => Ok(Self::Gif),
            "ico" => Ok(Self::Ico),
            "tiff" => Ok(Self::Tiff),
            _ => Err(format!("Unsupported image format: {}", s)),
        }
    }
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self::Png
    }
}

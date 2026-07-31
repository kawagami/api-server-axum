use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversionDirection {
    T2s,
    S2t,
}

#[derive(Deserialize)]
pub struct ConvertTextRequest {
    pub text: String,
    pub direction: ConversionDirection,
}

#[derive(Serialize)]
pub struct ConvertTextResponse {
    pub original_text: String,
    pub converted_text: String,
}

use serde::{Deserialize, Serialize};

// 常數定義
pub const DEFAULT_PASSWORD_COUNT: u8 = 1;
pub const DEFAULT_PASSWORD_LENGTH: u8 = 8;
pub const DEFAULT_FULL: i64 = 28800;
pub const DEFAULT_REMAINING_TROOPS: i64 = 0;

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

#[derive(Deserialize, Debug)]
pub struct Troops {
    pub now: i64,
    #[serde[default="default_full"]]
    pub full: i64,
    #[serde[default="default_remaining_troops"]]
    pub remaining_troops: i64,
}

fn default_full() -> i64 {
    DEFAULT_FULL
}

fn default_remaining_troops() -> i64 {
    DEFAULT_REMAINING_TROOPS
}

#[derive(Serialize)]
pub struct CompleteTimeResponse {
    pub complete_time: String,
    pub minutes: i64,
}

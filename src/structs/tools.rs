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

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub description: String,
    pub category: String,
}

#[derive(Deserialize)]
pub struct UpdateSetting {
    pub value: String,
}

/// 批次更新 body：`{ "values": { key: value, … } }`
#[derive(Deserialize)]
pub struct UpdateSettings {
    pub values: std::collections::BTreeMap<String, String>,
}

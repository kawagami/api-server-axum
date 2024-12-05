use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FirebaseImage {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub files: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteImageRequest {
    pub file_name: String,
}

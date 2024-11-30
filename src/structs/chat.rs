use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    pub token: String,
}

#[derive(Deserialize)]
pub struct GetParams {
    pub limit: Option<i32>,
}

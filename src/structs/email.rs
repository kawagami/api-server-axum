use serde::Deserialize;

#[derive(Deserialize)]
pub struct EmailParams {
    pub subject: String,
    pub body: String,
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Stock {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ClosingPrice")]
    pub closing_price: String,
    #[serde(rename = "MonthlyAveragePrice")]
    pub monthly_average_price: String,
}

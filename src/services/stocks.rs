use crate::{
    errors::{AppError, RequestError},
    structs::stocks::StockRequest,
};
use reqwest::Client;
use scraper::{Html, Selector};

/// Parses HTML document to extract stock buyback information
///
/// # Arguments
/// * `html` - HTML content as a string
///
/// # Returns
/// A vector of StockRequest objects containing extracted stock information
pub fn parse_buyback_stock_raw_html(html: String) -> Vec<StockRequest> {
    let document = Html::parse_document(&html);

    // Define all selectors outside the loop
    let row_selector = Selector::parse("tr.odd, tr.even").unwrap_or_else(|e| {
        tracing::error!("Failed to parse row selector: {}", e);
        Selector::parse("tr").unwrap() // Fallback selector
    });

    let cell_selector = Selector::parse("td").unwrap_or_else(|e| {
        tracing::error!("Failed to parse cell selector: {}", e);
        Selector::parse("td").unwrap() // Should never fail
    });

    // Extract data from each row
    document
        .select(&row_selector)
        .filter_map(|row| {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            // Skip rows that don't have enough cells
            if cells.len() < 11 {
                return None;
            }

            // Extract required data, with better text handling
            let get_cell_text = |index: usize| -> String {
                cells
                    .get(index)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            };

            let stock_no = get_cell_text(1);
            let start_date = get_cell_text(9).replace("/", "");
            let end_date = get_cell_text(10).replace("/", "");

            // Skip records with missing data
            if stock_no.is_empty() || start_date.is_empty() || end_date.is_empty() {
                return None;
            }

            Some(StockRequest {
                stock_no,
                start_date,
                end_date,
            })
        })
        .collect()
}

/// 取得庫藏股列表頁面資訊 string
pub async fn get_buyback_stock_raw_html_string(
    reqewst_client: &Client,
    start_date: &str,
    end_date: &str,
) -> Result<String, AppError> {
    // Prepare form data
    let form_data = form_urlencoded::Serializer::new(String::new())
        .append_pair("encodeURIComponent", "1")
        .append_pair("step", "1")
        .append_pair("firstin", "1")
        .append_pair("off", "1")
        .append_pair("TYPEK", "sii")
        .append_pair("d1", start_date)
        .append_pair("d2", end_date)
        .append_pair("RD", "1")
        .finish();

    // Send POST request to get the data
    let response = reqewst_client
        .post("https://mopsov.twse.com.tw/mops/web/ajax_t35sc09")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_data)
        .send()
        .await?;

    // Check if request was successful
    if !response.status().is_success() {
        return Err(AppError::RequestError(RequestError::InvalidContent(
            "取資料失敗".to_string(),
        )));
    }

    Ok(response.text().await?)
}

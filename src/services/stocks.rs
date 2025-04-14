use crate::structs::stocks::StockRequest;
use scraper::{Html, Selector};

pub fn parse_document(html: String) -> Vec<StockRequest> {
    let document = Html::parse_document(&html);

    // Define selectors for the table rows
    let row_selector = Selector::parse("tr.odd, tr.even").unwrap();

    // Extract data from each row
    let mut records = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&Selector::parse("td").unwrap()).collect();

        // Skip rows that don't have enough cells
        if cells.len() < 11 {
            continue;
        }

        // Extract company code (2nd cell)
        let stock_no = cells[1].text().collect::<String>().trim().to_string();

        // Extract buyback period start date (9th cell)
        let start_date = cells[9].text().collect::<String>().trim().replace("/", "");

        // Extract buyback period end date (10th cell)
        let end_date = cells[10].text().collect::<String>().trim().replace("/", "");

        // Create and store record
        let record = StockRequest {
            stock_no,
            start_date,
            end_date,
        };

        records.push(record);
    }

    records
}

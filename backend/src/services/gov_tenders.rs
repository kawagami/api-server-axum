use crate::{
    errors::AppError,
    repositories::gov_tenders as repo,
    structs::gov_tenders::{GovTender, GovTenderListQuery, GovTenderPaginatedResponse, NewGovTender},
};
use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

/// g0v 政府電子採購網 API（ronnywang 維護，舊網址 pcc.g0v.ronny.tw 已轉址至此）
const SEARCH_URL: &str = "https://pcc-api.openfun.app/api/searchbytitle";
/// 通知信最多列出的標案筆數
const EMAIL_MAX_ITEMS: usize = 50;

pub async fn list(
    pool: &Pool<Postgres>,
    query: &GovTenderListQuery,
    limit: i64,
    offset: i64,
) -> Result<GovTenderPaginatedResponse, AppError> {
    let total = repo::count(pool, query).await?;
    let data = repo::list(pool, query, limit, offset).await?;
    Ok(GovTenderPaginatedResponse { data, total })
}

/// 以關鍵字搜尋標案公告（第 1 頁 100 筆，依公告日新到舊，足以涵蓋每日增量）
pub async fn fetch_by_keyword(
    client: &Client,
    keyword: &str,
) -> Result<Vec<NewGovTender>, AppError> {
    let text = client
        .get(SEARCH_URL)
        .query(&[("query", keyword), ("page", "1")])
        .send()
        .await?
        .text()
        .await?;
    parse_records(&text, keyword)
}

#[derive(Deserialize)]
struct ApiResponse {
    records: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct ApiRecord {
    date: u32,
    filename: String,
    brief: ApiBrief,
    job_number: String,
    unit_id: String,
    unit_name: String,
}

#[derive(Deserialize)]
struct ApiBrief {
    #[serde(rename = "type")]
    kind: String,
    title: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    companies: ApiCompanies,
}

#[derive(Deserialize, Default)]
struct ApiCompanies {
    #[serde(default)]
    names: Vec<String>,
}

/// 解析搜尋 API 回應；單筆欄位缺漏只跳過該筆，不影響整批
fn parse_records(text: &str, keyword: &str) -> Result<Vec<NewGovTender>, AppError> {
    let resp: ApiResponse = serde_json::from_str(text)?;
    let tenders = resp
        .records
        .into_iter()
        .filter_map(|v| serde_json::from_value::<ApiRecord>(v).ok())
        .filter_map(|r| {
            let date = parse_date(r.date)?;
            Some(NewGovTender {
                detail_url: detail_url(r.date, &r.filename),
                filename: r.filename,
                date,
                tender_type: r.brief.kind,
                title: r.brief.title,
                category: r.brief.category,
                unit_id: r.unit_id,
                unit_name: r.unit_name,
                job_number: r.job_number,
                companies: r.brief.companies.names,
                keyword: keyword.to_string(),
            })
        })
        .collect();
    Ok(tenders)
}

/// API 的日期是 20260703 這種整數
fn parse_date(v: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt((v / 10000) as i32, v / 100 % 100, v % 100)
}

/// 官方公告頁連結（web.pcc.gov.tw 依公告日 + 檔名轉導）
fn detail_url(date: u32, filename: &str) -> String {
    format!(
        "https://web.pcc.gov.tw/prkms/tender/common/noticeDate/redirectPublic?ds={date}&fn={filename}.xml"
    )
}

pub fn compose_email(rows: &[GovTender]) -> (String, String) {
    let subject = format!("政府採購網有 {} 筆新標案公告", rows.len());

    let mut body = format!("追蹤關鍵字命中 {} 筆新標案公告：\n\n", rows.len());
    for r in rows.iter().take(EMAIL_MAX_ITEMS) {
        body.push_str(&format!(
            "・{}［{}］{} — {}\n　{}\n",
            r.date.format("%Y-%m-%d"),
            r.tender_type,
            r.title,
            r.unit_name,
            r.detail_url
        ));
    }
    if rows.len() > EMAIL_MAX_ITEMS {
        body.push_str(&format!(
            "\n…其餘 {} 筆請至後台標案頁查看。\n",
            rows.len() - EMAIL_MAX_ITEMS
        ));
    }
    (subject, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "query": "\"網站\"", "page": 1, "total_records": 2, "total_pages": 1,
        "records": [
            {
                "date": 20260703, "filename": "TIQ-4-71060709",
                "brief": {
                    "type": "公開取得報價單或企劃書公告",
                    "title": "網站設計建置暨維護作業案",
                    "category": "勞務類842-軟體執行服務",
                    "companies": {"ids": ["24746612"], "names": ["良知股份有限公司"]}
                },
                "job_number": "1150723", "unit_id": "3.97.9.12", "unit_name": "高雄市鳳山區忠孝國民小學"
            },
            {
                "date": 20260701, "filename": "NAI-1-71218651",
                "brief": {
                    "type": "無法決標公告",
                    "title": "115年「網站弱掃軟體授權續約更新」採購案",
                    "companies": {"ids": [], "names": []}
                },
                "job_number": "115D26", "unit_id": "3.87.14", "unit_name": "臺中市政府衛生局"
            },
            { "date": 20269999, "filename": "BAD-DATE", "brief": {"type": "x", "title": "x"}, "job_number": "1", "unit_id": "1", "unit_name": "x" },
            { "date": 20260101 }
        ]
    }"#;

    #[test]
    fn parse_records_skips_bad_rows() {
        let rows = parse_records(SAMPLE, "網站").unwrap();
        assert_eq!(rows.len(), 2);

        let first = &rows[0];
        assert_eq!(first.filename, "TIQ-4-71060709");
        assert_eq!(first.date, NaiveDate::from_ymd_opt(2026, 7, 3).unwrap());
        assert_eq!(first.tender_type, "公開取得報價單或企劃書公告");
        assert_eq!(first.category.as_deref(), Some("勞務類842-軟體執行服務"));
        assert_eq!(first.companies, vec!["良知股份有限公司"]);
        assert_eq!(first.keyword, "網站");
        assert_eq!(
            first.detail_url,
            "https://web.pcc.gov.tw/prkms/tender/common/noticeDate/redirectPublic?ds=20260703&fn=TIQ-4-71060709.xml"
        );

        // 無 category 的公告仍收，category 為 None
        assert_eq!(rows[1].category, None);
        assert!(rows[1].companies.is_empty());
    }

    #[test]
    fn parse_records_rejects_non_json() {
        assert!(parse_records("<html>oops</html>", "網站").is_err());
    }

    #[test]
    fn parse_date_valid_and_invalid() {
        assert_eq!(parse_date(20260703), NaiveDate::from_ymd_opt(2026, 7, 3));
        assert_eq!(parse_date(20261301), None);
    }
}

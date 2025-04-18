use crate::errors::{AppError, RequestError};
use reqwest::{Client, Method};
use std::collections::HashMap;

/// 通用的網頁 HTML 獲取函數
pub async fn get_raw_html_string(
    request_client: &Client,
    url: &str,
    method: Method,
    headers: Option<HashMap<String, String>>,
    form_data_pairs: Option<Vec<(&str, &str)>>,
) -> Result<String, AppError> {
    // 建立基本請求
    let mut request_builder = request_client.request(method, url);

    // 添加自訂標頭
    if let Some(headers_map) = headers {
        request_builder = headers_map
            .iter()
            .fold(request_builder, |builder, (key, value)| {
                builder.header(key, value)
            });
    }

    // 添加表單數據（若有提供）
    if let Some(form_pairs) = form_data_pairs {
        let form_data = form_pairs
            .iter()
            .fold(
                form_urlencoded::Serializer::new(String::new()),
                |mut serializer, &(key, value)| {
                    serializer.append_pair(key, value);
                    serializer
                },
            )
            .finish();

        request_builder = request_builder
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_data);
    }

    // 發送請求獲取數據
    let response = request_builder.send().await?;

    // 檢查請求是否成功
    if !response.status().is_success() {
        return Err(AppError::RequestError(RequestError::InvalidContent(
            format!("獲取 {} 頁面數據失敗，狀態碼: {}", url, response.status()),
        )));
    }

    Ok(response.text().await?)
}

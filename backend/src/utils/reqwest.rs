use crate::errors::{AppError, RequestError};
use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

fn build_request<'a>(
    client: &Client,
    method: Method,
    url: &str,
    headers: Option<HashMap<String, String>>,
    form_data_pairs: Option<Vec<(&'a str, &'a str)>>,
    json_body: Option<&serde_json::Value>,
) -> RequestBuilder {
    let mut builder = client.request(method, url);

    if let Some(headers_map) = headers {
        builder = headers_map
            .iter()
            .fold(builder, |b, (key, value)| b.header(key, value));
    }

    if let Some(form_pairs) = form_data_pairs {
        builder = builder.form(&form_pairs);
    } else if let Some(json) = json_body {
        builder = builder.json(json);
    }

    builder
}

/// 通用的網頁 HTML 獲取函數
pub async fn get_raw_html_string(
    request_client: &Client,
    url: &str,
    method: Method,
    headers: Option<HashMap<String, String>>,
    form_data_pairs: Option<Vec<(&str, &str)>>,
) -> Result<String, AppError> {
    let response = build_request(request_client, method, url, headers, form_data_pairs, None)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(RequestError::InvalidContent(format!(
            "獲取 {} 頁面數據失敗，狀態碼: {}",
            url,
            response.status()
        ))
        .into());
    }

    Ok(response.text().await?)
}

/// 通用的 JSON 資料獲取函數
pub async fn get_json_data<T>(
    request_client: &Client,
    url: &str,
    method: Method,
    headers: Option<HashMap<String, String>>,
    form_data_pairs: Option<Vec<(&str, &str)>>,
    json_body: Option<&serde_json::Value>,
) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    let response = build_request(
        request_client,
        method,
        url,
        headers,
        form_data_pairs,
        json_body,
    )
    .send()
    .await?;

    if !response.status().is_success() {
        return Err(RequestError::InvalidContent(format!(
            "獲取 {} 數據失敗，狀態碼: {}",
            url,
            response.status()
        ))
        .into());
    }

    Ok(response.json::<T>().await?)
}

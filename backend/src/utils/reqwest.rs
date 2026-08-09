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

/// 送出請求，對**連線階段**的暫時性失敗重試。
///
/// 存在的理由（2026-08-09）：這台 VPS 的 IPv6 是半殘的（有 stack、有 link-local，沒有
/// 全域位址也沒有 `::/0` 路由），於是對外連線偶爾會先試 AAAA 再撞
/// `Cannot assign requested address`。7 天內 4 次，每次都是**單發**——
/// 而使用者用 Google 登入的那條路徑一次失敗就直接回 502。
///
/// 重試只涵蓋 `is_connect()` / `is_timeout()`，也就是**請求還沒送達對方**的失敗。
/// 對方已經收到並回了狀態碼的，一律原樣回傳 —— 那不是抖動，重試只會放大問題。
///
/// ⚠ 這是**症狀防護**，不是根因修復。根因是宿主機的 IPv6 設定，見
/// `deploy/docker-compose.yml` 的 backend sysctls 註解。
pub async fn send_retrying(builder: RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
    /// 總嘗試次數（含第一次）
    const ATTEMPTS: u32 = 3;
    /// 每次退避的基數，實際等待是 base × 第幾次（200ms / 400ms）
    const BACKOFF_MS: u64 = 200;

    let mut attempt = 1;
    loop {
        // body 不可重放（串流）時沒有重試的餘地，直接送一次。
        // 目前所有呼叫端都是 form / 無 body，走不到這條。
        let Some(cloned) = builder.try_clone() else {
            return builder.send().await;
        };

        match cloned.send().await {
            Ok(response) => return Ok(response),
            Err(e) => {
                let transient = e.is_connect() || e.is_timeout();
                if attempt >= ATTEMPTS || !transient {
                    return Err(e);
                }
                tracing::warn!(
                    "對外請求暫時性失敗（第 {}/{} 次），{}ms 後重試: {}",
                    attempt,
                    ATTEMPTS,
                    BACKOFF_MS * attempt as u64,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_millis(
                    BACKOFF_MS * attempt as u64,
                ))
                .await;
                attempt += 1;
            }
        }
    }
}

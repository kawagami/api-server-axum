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
/// 存在的理由（2026-08-09）：對外連線偶爾在 connect 階段撞
/// `Cannot assign requested address`（AAAA 位址接不上），每次都是**單發**——
/// 而使用者用 Google 登入的那條路徑一次失敗就直接回 502。
///
/// 為什麼「重試」剛好是對的解：hyper 的 happy eyeballs 只在**解析結果同時有 A 與 AAAA**
/// 時才會 v6 失敗後退回 v4；答案只有 AAAA 時 fallback 是空的，第一個錯誤直接吐出來。
/// 而那種 AAAA-only 的答案來自 Docker 內嵌 DNS 的冷快取（容器剛重啟、A 與 AAAA 兩個
/// 上游查詢掉了一個），**下一次解析就正常** —— 所以重試會重新 resolve 並接上。
/// 完整推導見 `deploy/README.md` 的「對外連線偶發 connect 失敗」。
///
/// 重試只涵蓋 `is_connect()` / `is_timeout()`，也就是**請求還沒送達對方**的失敗。
/// 對方已經收到並回了狀態碼的，一律原樣回傳 —— 那不是抖動，重試只會放大問題。
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
                    error_chain(&e)
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

/// 把整條 source chain 串成一行。
///
/// `reqwest::Error` 的 Display 只有最外層 —— `error sending request for url (…)`，
/// 唯一有用的 errno 全在底下。2026-08-09 追這個問題時，這行 WARN 什麼都看不出來，
/// 答案是從 `errors.rs` 用 `{:?}` 印進 `fields.self` 的那筆 ERROR 才撈到的
/// （`… <- tcp connect error <- Cannot assign requested address (os error 99)`）。
/// 而重試成功的請求根本不會產生那筆 ERROR，等於整段線索消失。
fn error_chain(err: &dyn std::error::Error) -> String {
    use std::fmt::Write;

    let mut out = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        let _ = write!(out, " <- {cause}");
        source = cause.source();
    }
    out
}

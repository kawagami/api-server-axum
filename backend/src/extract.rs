//! 全站的 extractor —— `Json` / `Query` / `Path` 一律用這裡的，**不要直接用 axum 的**。
//!
//! 差別只有一件事：rejection 轉成 `AppError`。
//!
//! axum 原生 extractor 被拒絕時會直接回一段**純文字**（`Failed to deserialize the JSON
//! body into the target type: missing field \`occurred_at\``），那條路徑完全不經過
//! `AppError::into_response`，於是：
//! - body 形狀跟全站其他錯誤不一樣（客戶端要 parse 兩種格式）
//! - **沒有 `request_id`**，使用者回報「送出沒反應」時對不到任何 log
//! - **不落 log**：`logs` 表裡查不到這個請求失敗過
//!
//! 而少填一個欄位這種 400/422，正是回報量最大的那類問題。
//!
//! ⚠️ **狀態碼一律沿用 axum 原本回的**（`RequestError::Rejection` 帶著 `status` 走），
//! 只換 body。逐一映射到既有 variant 會在 axum 新增 rejection 種類時悄悄改掉狀態碼。
//!
//! ⚠️ **`clippy.toml` 把 `axum::Json` / `axum::extract::Query` / `axum::extract::Path`
//! 列為禁用型別**，CI 跑的是 `cargo clippy -- -D warnings`，所以用錯會直接紅燈。
//! 這是刻意的：漏用這裡的版本沒有任何執行期徵兆，只能靠 lint 擋。
//!
//! 蓋不到的仍有兩處（那兩個不是 extractor）：`RequestBodyLimitLayer` 的 413、
//! Router 對不上 method 的 405。

// 這裡就是那個包裝層，本來就得直接碰 axum 的原生型別
#![allow(clippy::disallowed_types)]

use crate::errors::{normalize_error_response, AppError};
use axum::{
    extract::{FromRequest, FromRequestParts, OptionalFromRequest, Request},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Serialize};

/// rejection → `AppError`。
///
/// 5xx 的 rejection 只有 `PathRejection::MissingPathParams`（handler 的 `Path<T>` 與
/// 路由上的 `{param}` 對不起來 —— 那是**我們寫錯**，不是使用者送錯），所以轉成
/// `SystemError`：它才會記 ERROR、才會被當成要修的 bug。
fn to_app_error<R>(rejection: R) -> AppError
where
    R: IntoResponse + std::fmt::Display,
{
    let message = rejection.to_string();
    let status = rejection.into_response().status();
    normalize_error_response(status, message)
}

/// `axum::Json` 的替身：extractor 的 rejection 走 `AppError`，回應行為完全相同。
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = <axum::Json<T> as FromRequest<S>>::from_request(req, state)
            .await
            .map_err(to_app_error)?;
        Ok(Self(value))
    }
}

/// `Option<Json<T>>`（body 可有可無的端點，如 `POST /member/vocab/runs`）走的是
/// 另一條 trait，不 impl 這個的話那種 handler 會編不過
impl<T, S> OptionalFromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
        match <axum::Json<T> as OptionalFromRequest<S>>::from_request(req, state).await {
            Ok(Some(axum::Json(value))) => Ok(Some(Self(value))),
            Ok(None) => Ok(None),
            Err(rejection) => Err(to_app_error(rejection)),
        }
    }
}

/// `Json` 同時是回應型別，這個 impl 讓 handler 的回傳型別不必換回 axum 的
impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

/// `axum::extract::Query` 的替身
pub struct Query<T>(pub T);

impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(value) = axum::extract::Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(to_app_error)?;
        Ok(Self(value))
    }
}

/// `axum::extract::Path` 的替身
pub struct Path<T>(pub T);

impl<T, S> FromRequestParts<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(value) = axum::extract::Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(to_app_error)?;
        Ok(Self(value))
    }
}

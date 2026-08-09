// 這裡是錯誤 body 的產生點，用的是 axum::Json 的**回應**能力，
// 不涉及 `crate::extract` 要修的 extractor rejection（那邊反過來依賴這裡，繞回來會循環）
#![allow(clippy::disallowed_types)]

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

// 用於 API 回應的錯誤結構
#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    // 對應該次請求的追蹤 id（同時放在 x-request-id header），使用者回報錯誤時可據此撈 log
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

// 變體統一 *Error 後綴是本專案慣例，比去掉後綴的裸名（Connection / Request…）可讀
#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum AppError {
    // HTTP 相關錯誤
    #[error("服務連接失敗: {0}")]
    ConnectionError(#[source] anyhow::Error),

    // 請求處理錯誤
    #[error("請求處理失敗: {0}")]
    RequestError(#[from] RequestError),

    // 認證相關錯誤
    #[error("認證失敗: {0}")]
    AuthError(#[from] AuthError),

    // 系統錯誤
    #[error("系統錯誤: {0}")]
    SystemError(#[from] SystemError),
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("multipart 欄位處理失敗: {0}")]
    MultipartError(#[source] anyhow::Error),

    #[error("無效的請求內容: {0}")]
    InvalidContent(String),

    #[error("驗證失敗: {0}")]
    UnprocessableContent(String),

    #[error("資源衝突: {0}")]
    Conflict(String),

    #[error("儲存空間不足: {0}")]
    InsufficientStorage(String),

    #[error("找不到資源")]
    NotFound,

    /// 限流。有這個 variant，rate_limit middleware 才不必自己組 JSON —— 否則被限流的
    /// 使用者拿到的是唯一一種沒有 request_id 的錯誤 body，回報問題時對不到 log。
    #[error("{0}")]
    TooManyRequests(String),

    /// 請求**還沒到 handler** 就被拒的錯誤：extractor rejection（見 `extract.rs`）
    /// 與 layer / Router 直接吐的回應（413、405，見 `middleware/error_shape.rs`）。
    ///
    /// **狀態碼沿用原本那個**，只把 body 換成全站統一形狀。逐一映射到既有 variant 的話，
    /// 哪天多一種我們沒對到的拒絕原因，狀態碼就會被悄悄改掉 —— 帶著 status 走是唯一
    /// 不會漂移的做法。
    /// （5xx 在轉換函式那層就變成 `SystemError`，所以這個 variant 恆為 4xx。）
    #[error("{message}")]
    Rejection { status: StatusCode, message: String },
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("缺少認證 Token")]
    MissingToken,

    #[error("無效的認證標頭")]
    InvalidHeader,

    #[error("Token 已過期")]
    TokenExpired,

    #[error("無效的 Token")]
    InvalidToken,

    #[error("未授權的存取")]
    Unauthorized,

    #[error("權限不足")]
    Forbidden,

    #[error("使用者不存在")]
    UserNotFound,

    #[error("密碼驗證失敗")]
    InvalidPassword,

    // 登入失敗統一回此訊息，不區分帳號不存在/密碼錯誤，防帳號枚舉
    #[error("帳號或密碼錯誤")]
    InvalidCredentials,

    // WebAuthn 挑戰過期/驗證失敗/憑證不存在統一回此訊息，不外洩細節
    #[error("Passkey 驗證失敗")]
    WebauthnFailed,
}

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("內部錯誤: {0}")]
    Internal(String),

    #[error("資料庫錯誤")]
    Database(#[source] sqlx::Error),

    #[error("Redis 錯誤")]
    Redis(#[source] redis::RedisError),

    #[error("JSON 處理錯誤")]
    Json(#[source] serde_json::Error),

    #[error("時間解析錯誤")]
    TimeParse(#[source] chrono::ParseError),
}

/// SystemError 對外的統一訊息。
///
/// `SystemError::Internal(String)` 的 Display 會把內部字串原樣帶出去，而那些字串常含
/// 檔案系統路徑、errno、上游回應片段（`serve file failed: {e}`、`儲存圖片失敗: {e}`，
/// 以及 anyhow / reqwest 的 blanket 轉換）。在 `error_response` 這一層統一換成通用
/// 訊息，比逐一修每個 `format!` 徹底 —— 新增的 Internal 也自動被涵蓋。
///
/// 細節不會消失：`IntoResponse` 已經用 `tracing::error!(?self, …)` 記下完整內容，
/// 對外回應帶著 `request_id`，要查就用它對 log。
const SYSTEM_ERROR_PUBLIC_MESSAGE: &str = "伺服器內部錯誤，請稍後再試";

impl AppError {
    fn error_response(&self) -> ErrorResponse {
        let status = self.status_code();

        ErrorResponse {
            code: status.as_u16(),
            // SystemError 的內部細節只進 log，不出站
            message: match self {
                Self::SystemError(_) => SYSTEM_ERROR_PUBLIC_MESSAGE.to_string(),
                _ => self.to_string(),
            },
            details: self.error_details(),
            request_id: crate::middleware::request_id::current_request_id(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConnectionError(_) => StatusCode::BAD_GATEWAY,
            Self::RequestError(err) => match err {
                RequestError::MultipartError(_) => StatusCode::BAD_REQUEST,
                RequestError::InvalidContent(_) => StatusCode::BAD_REQUEST,
                RequestError::UnprocessableContent(_) => StatusCode::UNPROCESSABLE_ENTITY,
                RequestError::Conflict(_) => StatusCode::CONFLICT,
                RequestError::InsufficientStorage(_) => StatusCode::INSUFFICIENT_STORAGE,
                RequestError::NotFound => StatusCode::NOT_FOUND,
                RequestError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
                RequestError::Rejection { status, .. } => *status,
            },
            Self::AuthError(err) => match err {
                AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                AuthError::InvalidHeader => StatusCode::BAD_REQUEST,
                AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
                AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
                AuthError::Forbidden => StatusCode::FORBIDDEN,
                AuthError::UserNotFound => StatusCode::UNAUTHORIZED,
                AuthError::InvalidPassword => StatusCode::UNAUTHORIZED,
                AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                AuthError::WebauthnFailed => StatusCode::UNAUTHORIZED,
            },
            Self::SystemError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_details(&self) -> Option<String> {
        if cfg!(debug_assertions) {
            Some(format!("{:#?}", self))
        } else {
            None
        }
    }
}

/// 422 的簡寫：驗證程式碼裡 `RequestError::UnprocessableContent(msg).into()` 出現頻率最高，
/// 每個 service 各寫一份私有 helper 是重複的來源（收斂前 `services/{invoices,lotto_tickets,
/// app_settings}.rs` 各一份，兩份吃 `&str`、一份吃 `String`，簽名還不一致）。
///
/// 吃 `impl Into<String>` 讓字面值與 `format!` 兩種呼叫端都不必在呼叫處補 `.into()`。
pub fn unprocessable(msg: impl Into<String>) -> AppError {
    AppError::RequestError(RequestError::UnprocessableContent(msg.into()))
}

/// 「沒帶票 / 票過期」這類日常 401 —— 記 debug。
///
/// 反過來說，留在 WARN 的是**帶著身分卻被擋下**的那些：`Forbidden`（權限不足）、
/// `InvalidPassword` / `InvalidCredentials`（登入失敗）、`WebauthnFailed`、
/// `UserNotFound`（token 有效但帳號已刪 —— 撤銷沒撤乾淨的徵兆）。那幾種每一筆
/// 都值得看，且可以用 `request_id` 對回 `admin_audit_logs` 查是誰。
fn is_routine_auth(err: &AuthError) -> bool {
    matches!(
        err,
        AuthError::MissingToken
            | AuthError::TokenExpired
            | AuthError::InvalidToken
            | AuthError::InvalidHeader
            | AuthError::Unauthorized
    )
}

/// 「量大又沒有診斷價值」的 4xx —— 記 debug。
///
/// 反過來說，留在 WARN 的是**請求進得來、卻在業務規則上被擋下**的那些：
/// `UnprocessableContent`（驗證失敗）、`Conflict`（資源衝突）、`InsufficientStorage`、
/// `MultipartError` / `InvalidContent`（上傳與請求體壞掉）。
///
/// 為什麼要提到 WARN：原本整個 `RequestError` 都是 debug，而生產的落地門檻是 WARN，
/// 於是所有 4xx 在 `logs` 表**零紀錄**。使用者回報最多的「按了沒反應 / 存不進去」正是
/// 422 與 409。
///
/// 兩個例外：
/// - `NotFound` —— 爬蟲掃站（/wp-admin 之類）與 `with_feature` 關閉功能都走這條，
///   跟 `TokenExpired` 同一種噪音。
/// - `TooManyRequests` —— 被擋下的請求**本來就是連續一整串**，每筆一列等於讓攻擊者
///   決定 `logs` 表的寫入量。這條的訊號已經由 `middleware/rate_limit.rs` 記了：
///   只在 `count == max + 1`（剛超過）那一刻一筆 WARN，帶 scope 與 ip。
fn is_routine_request(err: &RequestError) -> bool {
    matches!(
        err,
        RequestError::NotFound | RequestError::TooManyRequests(_)
    )
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_response = self.error_response();

        // Log the error with different levels based on severity
        match &self {
            AppError::SystemError(_) => {
                tracing::error!(?self, "System error occurred");
            }
            AppError::ConnectionError(_) => {
                tracing::error!(?self, "Connection error occurred");
            }
            // 憑證過期 / 沒帶 token 是客戶端的**常態**（前端 token 只有 1 小時、
            // kawa-logs CLI 也是 401 才續期），全部記 WARN 的話它們會淹掉真正的
            // 安全訊號 —— 實測 14 天內這條佔了 logs 表 WARN 的三成，內容幾乎都是
            // TokenExpired / MissingToken。所以只有「有身分但不該過」的那幾種留在 WARN。
            AppError::AuthError(err) if is_routine_auth(err) => {
                tracing::debug!(?self, "Authentication error occurred");
            }
            AppError::AuthError(_) => {
                tracing::warn!(?self, "Authentication error occurred");
            }
            // 4xx 分兩級，理由同上面的 AuthError：只有 404 那種掃站噪音留 debug，
            // 其餘（422 / 409 / 429 / 400 / 507）進 WARN，否則生產的 logs 表看不到任何 4xx。
            AppError::RequestError(err) if is_routine_request(err) => {
                tracing::debug!(?self, "Request error occurred");
            }
            AppError::RequestError(_) => {
                tracing::warn!(?self, "Request error occurred");
            }
        }

        (self.status_code(), Json(error_response)).into_response()
    }
}

/// 讀取待正規化錯誤 body 的上限。錯誤 body 一律很短（`length limit exceeded` 那種），
/// 這個上限只是防呆 —— 真有超過的就當空的，用狀態碼的標準說明頂上。
pub const MAX_ERROR_BODY: usize = 8 * 1024;

/// 把「不是 `AppError` 產生的」錯誤狀態＋文字轉成統一形狀（見 `middleware/error_shape.rs`
/// 與 `extract.rs`）。5xx 轉成 `SystemError` —— 那是我們的 bug，要記 ERROR。
pub fn normalize_error_response(status: StatusCode, message: String) -> AppError {
    if status.is_server_error() {
        return AppError::SystemError(SystemError::Internal(message));
    }
    RequestError::Rejection { status, message }.into()
}

/// handler panic 的回應（給 `CatchPanicLayer::custom` 用）。
///
/// 沒有這層的話 panic 只會讓連線被切斷：client 看到的是 network error 而不是 500、
/// `logs` 表零筆、request span 沒有結束、`admin_audit_logs` 也不會有那筆（audit 在
/// `next.run` 之後才寫）。唯一的痕跡是 runtime 預設 panic hook 印在 stderr 的那行
/// —— 它不經過 tracing，而生產 image 無 shell，只能靠 `docker logs` 撈。
/// job 那側早就補上了同樣的防護（`scheduler.rs` 把 `job.run` 包進自己的 spawn 檢查
/// `is_panic`），請求路徑一直沒有。
///
/// 回應形狀走 `AppError`，所以自動帶 `request_id`、細節不外洩（統一的 500 訊息）。
pub fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> Response {
    // panic payload 只有 &str / String 兩種常見型別，取不到就留個明確字串
    let detail = err
        .downcast_ref::<&'static str>()
        .map(|s| (*s).to_string())
        .or_else(|| err.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic payload".to_string());

    // 這行才是查得到的那筆：`?q=panic` 能直接撈出來，位置資訊在 stderr 的 hook 輸出裡
    tracing::error!(panic = %detail, "handler panicked");

    AppError::SystemError(SystemError::Internal(format!("handler panic: {detail}"))).into_response()
}

// 便利函數
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::SystemError(SystemError::Internal(err.to_string()))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => RequestError::NotFound.into(),
            e => Self::SystemError(SystemError::Database(e)),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::SystemError(SystemError::Json(err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() || err.is_connect() {
            Self::ConnectionError(err.into())
        } else if err.is_decode() {
            RequestError::InvalidContent(err.to_string()).into()
        } else {
            Self::SystemError(SystemError::Internal(err.to_string()))
        }
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        Self::SystemError(SystemError::TimeParse(err))
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        Self::SystemError(SystemError::Redis(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_format() {
        let error = AppError::AuthError(AuthError::MissingToken);
        let response = error.error_response();

        assert_eq!(response.code, 401);
        assert_eq!(response.message, "認證失敗: 缺少認證 Token");
    }

    /// SystemError 的內部字串常含檔案路徑 / errno / 上游回應片段，不可出站。
    /// 細節只進 log，客戶端拿 request_id 去對。
    #[test]
    fn system_error_detail_never_reaches_client() {
        let leaky = AppError::SystemError(SystemError::Internal(
            "serve file failed: /srv/kawa/torrents/abc/secret.mkv (os error 13)".to_string(),
        ));
        let response = leaky.error_response();

        assert_eq!(response.code, 500);
        assert_eq!(response.message, SYSTEM_ERROR_PUBLIC_MESSAGE);
        assert!(!response.message.contains("/srv/kawa"));
        assert!(!response.message.contains("os error"));
    }

    #[test]
    fn test_status_codes() {
        let auth_error = AppError::AuthError(AuthError::MissingToken);
        assert_eq!(auth_error.status_code(), StatusCode::UNAUTHORIZED);

        let system_error = AppError::SystemError(SystemError::Internal("test".to_string()));
        assert_eq!(
            system_error.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}

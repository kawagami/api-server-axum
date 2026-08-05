use crate::{
    repositories::audit_logs::AuditEntry, state::AppState, structs::auth::AuthenticatedUser,
};
use axum::{
    body::Body,
    extract::{OriginalUri, Request, State},
    middleware::Next,
    response::Response,
};

pub async fn audit_log(State(state): State<AppState>, req: Request, next: Next) -> Response<Body> {
    // auth middleware（外層）已驗證並塞入 AuthenticatedUser，直接讀，不重複 decode JWT
    // audit log 的 user_email 欄現在存管理員顯示名（name）
    let user_email = req
        .extensions()
        .get::<AuthenticatedUser>()
        .map(|u| u.name.clone());

    let method = req.method().to_string();
    // audit 掛在 nest 內層，req.uri() 前綴已被剝掉；用 OriginalUri 取完整原始路徑
    let uri = req
        .extensions()
        .get::<OriginalUri>()
        .map(|o| &o.0)
        .unwrap_or_else(|| req.uri());
    let path = uri.path().to_string();
    let query = uri.query().map(ToString::to_string);
    // 必須在這裡取：request_id 是 task-local，一旦離開請求的 task（例如以前的
    // 每筆一個 spawn）就讀不到，那正是稽核列長年沒有 request_id 的原因。
    let request_id = crate::middleware::request_id::current_request_id();

    let response = next.run(req).await;

    if let Some(user_email) = user_email {
        let entry = AuditEntry {
            user_email,
            method,
            path,
            query,
            status_code: response.status().as_u16() as i16,
            request_id,
        };
        // 交給 `services::audit_logs::audit_writer` 批次寫入：請求路徑上不碰 DB。
        if let Err(e) = state.get_audit_tx().try_send(entry) {
            match e {
                // 佇列滿 = 寫入器追不上尖峰。刻意只記 debug：這條路徑上每丟一筆就記一則
                // WARN 的話，等於用「寫進 logs 表」來反應「DB 已經跟不上」。
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    tracing::debug!("audit 佇列已滿，丟棄一筆稽核紀錄");
                }
                // 寫入器已死 —— 這是 bug，之後所有稽核都會靜默消失，必須看得到
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    tracing::error!("audit 寫入器已關閉，稽核紀錄不再落地");
                }
            }
        }
    }

    response
}

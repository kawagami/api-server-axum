use crate::{state::AppState, structs::auth::AuthenticatedUser};
use axum::{body::Body, extract::{Request, State}, middleware::Next, response::Response};

pub async fn audit_log(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response<Body> {
    // auth middleware（外層）已驗證並塞入 AuthenticatedUser，直接讀，不重複 decode JWT
    let user_email = req
        .extensions()
        .get::<AuthenticatedUser>()
        .map(|u| u.email.clone());

    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(ToString::to_string);

    let response = next.run(req).await;

    if let Some(email) = user_email {
        let status_code = response.status().as_u16() as i16;
        let pool = state.get_pool().clone();
        tokio::spawn(async move {
            let _ = sqlx::query(
                "INSERT INTO admin_audit_logs (user_email, method, path, query, status_code) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(email)
            .bind(method)
            .bind(path)
            .bind(query)
            .bind(status_code)
            .execute(&pool)
            .await;
        });
    }

    response
}

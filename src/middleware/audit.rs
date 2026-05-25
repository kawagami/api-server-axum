use crate::state::AppState;
use axum::{body::Body, extract::{Request, State}, middleware::Next, response::Response};
use jsonwebtoken::{decode, DecodingKey, Validation};

pub async fn audit_log(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response<Body> {
    let user_email = extract_admin_email(&req, &state.get_config().jwt_secret);

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

fn extract_admin_email(req: &Request, jwt_secret: &str) -> Option<String> {

    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION)?;
    let token = auth_header.to_str().ok()?.split_whitespace().nth(1)?;

    let token_data = decode::<crate::structs::auth::Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;

    if token_data.claims.role != "admin" {
        return None;
    }

    Some(token_data.claims.sub)
}

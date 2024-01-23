use std::time::Duration;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::state::AppState;

mod v1;
mod v2;

pub async fn routes(state:AppState) -> Router {
    // let state = AppState {
    //     connection: get_connection().await,
    // };

    Router::new()
        .route("/ttest", get(using_connection_pool_extractor))
        .nest("/v1", v1::routes())
        .nest("/v2", v2::routes())
        .fallback(fallback)
        .with_state(state)
}

async fn fallback() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "api not found")
}

pub async fn get_connection(db_uri:String) -> Pool<Postgres> {
    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_uri)
        .await
        .expect("can't connect to database");

    pool
}

// async fn ttest(State(state): State<AppState>) -> String {
//     let result = sqlx::query("select 'hello world from pg' as result")
//         .fetch_one(&state.connection)
//         .await
//         .expect("fetch one fail");
//     format!("{:?}", result.try_get::<T, &str>("result"))
// }

// we can extract the connection pool with `State`
async fn using_connection_pool_extractor(
    State(state): State<AppState>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&state.connection)
        .await
        .map_err(internal_error)
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

use axum::{extract::State, routing::get, Router};
use futures::TryStreamExt;
use sqlx::{FromRow, Row};
use std::fmt::{Display, Formatter};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/test", get(test))
        // .route("/get_db_data", get(get_db_data))
}

pub async fn test() -> &'static str {
    "this is v1 test page"
}

// pub async fn get_db_data(State(state): State<AppState>) -> &'static str {
//     let mut rows = sqlx::query("SELECT * FROM users WHERE id = 1")
//         // .bind(1)
//         .fetch(&state.connection);

//     let result = "end";

//     while let Some(row) = rows.try_next().await.expect("try_next fail") {
//         // map the row into a user-defined domain type
//         let name: &str = row.try_get("name").expect("try_get fail");
//         println!("{name}");
//     }

//     result
// }

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"
            id: {},
            name: {},
        "#,
            self.id, self.name
        )
    }
}

#[derive(FromRow)]
struct User {
    pub id: i64,
    pub name: String,
    // pub hair_color: Option<String>,
}

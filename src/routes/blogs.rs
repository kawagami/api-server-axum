use crate::state::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::Serialize;
use sqlx::{postgres::PgRow, types::chrono::NaiveDateTime, Row};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Serialize, sqlx::FromRow)]
pub struct Blog {
    // #[sqlx(rename = "id")]
    id: i64,
    // #[sqlx(rename = "name")]
    name: String,
    // #[sqlx(rename = "short_content")]
    short_content: String,
    components: Vec<BlogComponent>,
    // #[sqlx(rename = "created_at")]
    created_at: NaiveDateTime,
    // #[sqlx(rename = "updated_at")]
    updated_at: NaiveDateTime,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct BlogComponent {
    // #[sqlx(rename = "content")]
    content: Option<String>,
    // #[sqlx(rename = "url")]
    url: Option<String>,
}

pub async fn get_blog(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<i32>,
) -> Result<Json<Blog>, (StatusCode, String)> {
    let pool = &state.lock().await.pool;
    let query = r#"
        SELECT
            b.id AS id,
            b."name" AS name,
            b.short_content AS short_content,
            bca.content AS content,
            bci.url AS url,
            b.created_at AS created_at,
            b.updated_at AS updated_at
        FROM
            blogs b
            LEFT JOIN blog_components bc ON b.id = bc.blog_id
            LEFT JOIN blog_component_articles bca ON bca.component_id = bc.id
            LEFT JOIN blog_component_images bci ON bci.component_id = bc.id
        WHERE
            b.id = $1;
    "#;
    let rows = sqlx::query(query)
        .bind(id)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    // 取特定資料不應該有空的狀況
    if rows.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "空的".to_string()));
    }

    Ok(Json(handle_blog(rows)))
}

fn handle_blog(rows: Vec<PgRow>) -> Blog {
    let id: i64 = rows[0].get("id");
    let name: String = rows[0].get("name");
    let short_content: String = rows[0].get("short_content");
    let created_at: NaiveDateTime = rows[0].get("created_at");
    let updated_at: NaiveDateTime = rows[0].get("updated_at");

    let mut components: Vec<BlogComponent> = Vec::with_capacity(rows.len());
    for row in rows {
        components.push(BlogComponent {
            content: row.get("content"),
            url: row.get("url"),
        });
    }

    Blog {
        id,
        name,
        short_content,
        components,
        created_at,
        updated_at,
    }
}

pub async fn get_blogs(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<Vec<Blog>>, (StatusCode, String)> {
    let pool = &state.lock().await.pool;
    let query = r#"
        SELECT
            b.id AS id,
            b."name" AS name,
            b.short_content AS short_content,
            bca.content AS content,
            bci.url AS url,
            b.created_at AS created_at,
            b.updated_at AS updated_at
        FROM
            blogs b
            LEFT JOIN blog_components bc ON b.id = bc.blog_id
            LEFT JOIN blog_component_articles bca ON bca.component_id = bc.id
            LEFT JOIN blog_component_images bci ON bci.component_id = bc.id
    "#;
    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    // 取 blogs 有可能是沒資料的
    let mut processing: BTreeMap<i64, Blog> = BTreeMap::default();

    for row in rows {
        let id: i64 = row.get("id");

        if processing.contains_key(&id) {
            let blog = processing
                .get_mut(&id)
                .expect("取得 BTreeMap 中的 blog 失敗");
            blog.components.push(BlogComponent {
                content: row.get("content"),
                url: row.get("url"),
            });
        } else {
            let name: String = row.get("name");
            let short_content: String = row.get("short_content");
            let components: Vec<BlogComponent> = vec![BlogComponent {
                content: row.get("content"),
                url: row.get("url"),
            }];
            let created_at: NaiveDateTime = row.get("created_at");
            let updated_at: NaiveDateTime = row.get("updated_at");
            let _ = processing.insert(
                id,
                Blog {
                    id,
                    name,
                    short_content,
                    components,
                    created_at,
                    updated_at,
                },
            );
        }
    }

    let asc_data = processing.into_values().collect::<Vec<Blog>>();

    Ok(Json(asc_data))
}

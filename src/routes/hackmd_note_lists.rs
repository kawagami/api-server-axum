use crate::state::AppStateV2;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;
use std::collections::HashSet;

#[derive(Serialize, sqlx::FromRow)]
pub struct HackmdNoteList {
    id: i64,
    is_public: bool,
    hackmd_note_lists_id: String,
    title: String,
    #[sqlx(rename = "createdAt")]
    created_at_hackmd: i64,
    #[sqlx(rename = "publishType")]
    publish_type: String,
    #[sqlx(rename = "publishedAt")]
    published_at: Option<i64>,
    permalink: Option<String>,
    #[sqlx(rename = "publishLink")]
    publish_link: String,
    #[sqlx(rename = "shortId")]
    short_id: String,
    #[sqlx(rename = "lastChangedAt")]
    last_changed_at: i64,
    #[sqlx(rename = "lastChangeUser")]
    last_change_user: sqlx::types::Json<LastChangeUser>,
    #[sqlx(rename = "userPath")]
    user_path: String,
    #[sqlx(rename = "teamPath")]
    team_path: Option<String>,
    #[sqlx(rename = "readPermission")]
    read_permission: String,
    #[sqlx(rename = "writePermission")]
    write_permission: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
struct LastChangeUser {
    name: String,
    photo: String,
    biography: Option<String>,
    #[sqlx(rename = "userPath")]
    user_path: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct HackmdNoteListAndTagString {
    id: i64,
    title: String,
    #[sqlx(rename = "publishLink")]
    publish_link: String,
    #[sqlx(rename = "lastChangedAt")]
    last_changed_at: i64,
    #[sqlx(rename = "readPermission")]
    read_permission: String,
    #[sqlx(rename = "tags")]
    tags: Option<String>,
}

#[derive(Serialize)]
pub struct HackmdNoteListAndCategories {
    id: i64,
    title: String,
    publish_link: String,
    last_changed_at: i64,
    read_permission: String,
    categories: HashSet<Option<String>>,
}

pub async fn get_note_list(
    State(state): State<AppStateV2>,
    Path(id): Path<i32>,
) -> Result<Json<HackmdNoteList>, (StatusCode, String)> {
    let pool = &state.get_pool().await;
    let query = "select * from hackmd_note_lists where id = $1";
    let result = sqlx::query_as::<_, HackmdNoteList>(query)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(result))
}

pub async fn get_all_note_lists(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<HackmdNoteListAndCategories>>, (StatusCode, String)> {
    let pool = &state.get_pool().await;
    let query = r#"
            SELECT
                nl.id,
                nl.title,
                nl."publishLink",
                nl."lastChangedAt",
                nl."readPermission",
                STRING_AGG(t.name, ',') AS tags
            FROM
                hackmd_note_lists nl
            LEFT JOIN hackmd_note_list_tag nlt ON nlt.note_list_id = nl.id
            LEFT JOIN hackmd_tags t ON nlt.tag_id = t.id
            GROUP BY
                nl.id, nl.title, nl."publishLink", nl."lastChangedAt", nl."readPermission"
            ORDER BY
                nl."lastChangedAt" DESC;
        "#;
    let records = sqlx::query_as::<_, HackmdNoteListAndTagString>(query)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    // 建立最後要回傳的資料容器
    let mut result: Vec<HackmdNoteListAndCategories> = Vec::new();

    // 將 tags 字串處理成不重複的 HashSet
    for record in records {
        let categories = parse(record.tags);
        let data = HackmdNoteListAndCategories {
            id: record.id,
            title: record.title,
            publish_link: record.publish_link,
            last_changed_at: record.last_changed_at,
            read_permission: record.read_permission,
            categories,
        };
        result.push(data);
    }

    Ok(Json(result))
}

fn parse(x: Option<String>) -> HashSet<Option<String>> {
    match x {
        None => HashSet::new(),
        Some(i) => {
            let mut set: HashSet<Option<String>> = HashSet::new();
            for x in i.split(",").collect::<Vec<&str>>() {
                set.insert(Some(x.to_string()));
            }

            set
        }
    }
}

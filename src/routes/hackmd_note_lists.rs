use crate::{
    state::AppStateV2,
    structs::hackmd::{HackmdNoteList, HackmdNoteListAndCategories, HackmdNoteListAndTagString},
};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use std::collections::HashSet;

pub async fn get_note_list(
    State(state): State<AppStateV2>,
    Path(id): Path<i32>,
) -> Result<Json<HackmdNoteList>, (StatusCode, String)> {
    let pool = &state.get_pool();
    let query = "select * from hackmd_note_lists where id = $1";
    let result: HackmdNoteList = sqlx::query_as(query)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(result))
}

pub async fn get_all_note_lists(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<HackmdNoteListAndCategories>>, (StatusCode, String)> {
    let pool = &state.get_pool();
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
    let records: Vec<HackmdNoteListAndTagString> = sqlx::query_as(query)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    // 建立最後要回傳的資料容器
    let mut result: Vec<HackmdNoteListAndCategories> = Vec::new();

    // 將 tags 字串處理成不重複的 HashSet
    for record in records {
        result.push(HackmdNoteListAndCategories {
            id: record.id,
            title: record.title,
            publish_link: record.publish_link,
            last_changed_at: record.last_changed_at,
            read_permission: record.read_permission,
            categories: parse(record.tags),
        });
    }

    Ok(Json(result))
}

fn parse(x: Option<String>) -> HashSet<String> {
    x.map(|i| i.split(',').map(|x| x.trim().to_string()).collect())
        .unwrap_or_default()
}

use crate::{state::AppStateV2, structs::hackmd::HackmdNoteListAndTag};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};

pub async fn get_all_note_lists(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<HackmdNoteListAndTag>>, (StatusCode, String)> {
    let pool = &state.get_pool();
    let query = r#"
            SELECT
                id,
                title,
                publish_link,
                last_changed_at,
                read_permission,
                tags
            FROM
                hackmd_posts
         	WHERE NOT (tags @> ARRAY['工作']) AND read_permission='guest'
            ORDER BY
                last_changed_at DESC;
        "#;
    let records: Vec<HackmdNoteListAndTag> = sqlx::query_as(query)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(records))
}

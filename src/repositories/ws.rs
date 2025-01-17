use crate::{state::AppStateV2, structs::ws::DbChatMessage};

pub async fn insert_chat_message(
    state: &AppStateV2,
    message_type: &str,
    to_type: &str,
    user_name: &str,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO chat_messages (message_type, to_type, user_name, message)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(message_type)
    .bind(to_type)
    .bind(user_name)
    .bind(message)
    .execute(&state.get_pool())
    .await?;

    Ok(())
}

pub async fn ws_message(
    state: &AppStateV2,
    limit: i32,
    before_id: Option<i32>,
) -> Result<Vec<DbChatMessage>, sqlx::Error> {
    let query = match before_id {
        Some(id) => sqlx::query_as(
            r#"
                    SELECT
                        id,
                        message_type,
                        to_type,
                        user_name,
                        message,
                        created_at
                    FROM
                        chat_messages
                    WHERE
                        id < $1
                    ORDER BY
                        id DESC
                    LIMIT
                        $2
                "#,
        )
        .bind(id)
        .bind(limit),
        None => sqlx::query_as(
            r#"
                    SELECT
                        id,
                        message_type,
                        to_type,
                        user_name,
                        message,
                        created_at
                    FROM
                        chat_messages
                    ORDER BY
                        id DESC
                    LIMIT
                        $1
                "#,
        )
        .bind(limit),
    };

    query.fetch_all(&state.get_pool()).await
}

use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::state::SharedState;

#[derive(Deserialize)]
pub struct Input {
    key: String,
    value: String,
}

pub async fn insert_one_data(
    State(state): State<SharedState>,
    Json(input): Json<Input>,
) -> Result<String, (StatusCode, String)> {
    let some_data = &mut state.write().unwrap().some_data;

    some_data.insert(input.key, input.value);

    Ok(format!("{:?}", some_data))
}

pub async fn read_state(State(state): State<SharedState>) -> Result<String, (StatusCode, String)> {
    let some_data = &state.read().unwrap().some_data;
    Ok(format!("{:?}", some_data))
}

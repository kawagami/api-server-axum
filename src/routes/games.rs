use crate::{games::snake::handler::ws_game_handler, state::AppStateV2};

use axum::{routing::get, Router};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/snake/{id}", get(ws_game_handler))
}

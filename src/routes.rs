mod blogs;
mod hackmd_note_lists;
mod handle_state;
mod products;
mod root;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::SharedState;

pub async fn app(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root::using_connection_pool_extractor))
        .route("/create_table", get(products::create_table))
        .route("/products", post(products::insert_one_product))
        .route(
            "/products/:id",
            get(products::get_product)
                .patch(products::update_product)
                .delete(products::delete_product),
        )
        .route("/note_lists/:id", get(hackmd_note_lists::get_note_list))
        .route("/note_lists", get(hackmd_note_lists::get_all_note_lists))
        .route("/blogs/:id", get(blogs::get_blog))
        .route(
            "/handle_state/insert_one",
            post(handle_state::insert_one_data),
        )
        .route("/handle_state/read_state", get(handle_state::read_state))
        .with_state(Arc::clone(&state))
}

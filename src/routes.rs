mod blogs;
mod hackmd_note_lists;
mod hackmd_note_list_tags;
mod handle_state;
mod images;
mod products;
mod root;

use std::sync::Arc;

use axum::{
    http::{header::CONTENT_TYPE, Method},
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

use crate::state::SharedState;

pub async fn app(state: SharedState) -> Router {
    let origins = [
        "http://localhost:5173".parse().unwrap(),
        "https://sg-vite.kawa.homes".parse().unwrap(),
    ];

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
        .route("/note_list_tags", get(hackmd_note_list_tags::get_all_note_list_tags))
        .route("/blogs/:id", get(blogs::get_blog))
        .route("/blogs", get(blogs::get_blogs))
        .route(
            "/handle_state/insert_one",
            post(handle_state::insert_one_data),
        )
        .route("/handle_state/read_state", get(handle_state::read_state))
        .nest_service(
            "/assets",
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/image404.png")),
        )
        .layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            //
            // pay attention that for some request types like posting content-type: application/json
            // it is required to add ".allow_headers([http::header::CONTENT_TYPE])"
            // or see this issue https://github.com/tokio-rs/axum/issues/849
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins)
                .allow_headers([CONTENT_TYPE]),
        )
        .with_state(Arc::clone(&state))
}

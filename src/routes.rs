mod hackmd_note_lists;
mod products;
mod root;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{Pool, Postgres};

pub async fn app(pool: Pool<Postgres>) -> Router {
    Router::new()
        .route(
            "/",
            get(root::using_connection_pool_extractor).post(root::using_connection_extractor),
        )
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
        .with_state(pool)
}

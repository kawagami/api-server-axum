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
        .with_state(pool)
}

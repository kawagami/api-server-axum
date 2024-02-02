use std::fmt;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::SharedState;

pub async fn create_table(
    State(state): State<SharedState>,
) -> Result<String, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    let query = r#"
        CREATE TABLE IF NOT EXISTS products(
            product_id serial PRIMARY KEY,
            product_name varchar(64) NOT NULL,
            description text,
            price DOUBLE PRECISION NOT NULL,
            stock_quantity integer NOT NULL,
            category_id integer
            -- created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            -- updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        );
    "#;
    let _row = sqlx::query_as(query)
        .fetch_one(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(format!("{:?}", "success"))
}

#[derive(Serialize, Deserialize, Default, sqlx::FromRow)]
pub struct Product {
    product_id: Option<i32>,
    product_name: String,
    description: Option<String>,
    price: f64,
    stock_quantity: i32,
    category_id: Option<i32>,
}

impl fmt::Display for Product {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(
            f,
            "product_id => {}\nproduct_name => {}\ndescription => {}\nprice => {}\nstock_quantity => {}\ncategory_id => {}",
            self.product_id.unwrap_or_default(),
            self.product_name,
            self.description.as_deref().unwrap_or_default(),
            self.price,
            self.stock_quantity,
            self.category_id.unwrap_or_default(),
        )
    }
}

pub async fn insert_one_product(
    State(state): State<SharedState>,
    Json(product): Json<Product>,
) -> Result<String, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    let row =
        sqlx::query_as::<_, Product>("INSERT INTO products (product_name, description, price, stock_quantity, category_id) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(product.product_name)
            .bind(product.description)
            .bind(product.price)
            .bind(product.stock_quantity)
            .bind(product.category_id)
            .fetch_one(pool)
            .await
            .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(format!("{}", row))
}

pub async fn get_product(
    State(state): State<SharedState>,
    Path(product_id): Path<i32>,
) -> Result<Json<Product>, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    let query = "select * from products where product_id = $1";
    let result = sqlx::query_as::<_, Product>(query)
        .bind(product_id)
        .fetch_one(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct UpdateProduct {
    product_name: Option<String>,
    description: Option<String>,
    price: Option<f64>,
    stock_quantity: Option<i32>,
    category_id: Option<i32>,
}

pub async fn update_product(
    State(state): State<SharedState>,
    Path(product_id): Path<i32>,
    Json(update_product): Json<UpdateProduct>,
) -> Result<Json<Product>, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    // 取得舊設定
    let query = "select * from products where product_id = $1";
    let mut original_product = sqlx::query_as::<_, Product>(query)
        .bind(product_id)
        .fetch_one(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    // 將有 input 的值把舊設定替換掉
    if let Some(product_name) = &update_product.product_name {
        original_product.product_name = product_name.to_string();
    }

    if let Some(description) = &update_product.description {
        original_product.description = Some(description.to_string());
    }

    if let Some(price) = &update_product.price {
        original_product.price = price.to_owned();
    }

    if let Some(stock_quantity) = &update_product.stock_quantity {
        original_product.stock_quantity = stock_quantity.to_owned();
    }

    if let Some(category_id) = &update_product.category_id {
        original_product.category_id = Some(category_id.to_owned());
    }

    let update_query = r#"
        UPDATE
            products
        SET
            product_name = $1,
            description = $2,
            price = $3,
            stock_quantity = $4,
            category_id = $5
        WHERE
            product_id = $6
        RETURNING
            *;
    "#;

    let result = sqlx::query_as::<_, Product>(update_query)
        .bind(original_product.product_name)
        .bind(original_product.description)
        .bind(original_product.price)
        .bind(original_product.stock_quantity)
        .bind(original_product.category_id)
        .bind(product_id)
        .fetch_one(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(result))
}

pub async fn delete_product(
    State(state): State<SharedState>,
    Path(product_id): Path<i32>,
) -> Result<String, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    let query = "DELETE FROM products WHERE product_id = $1";
    let _result = sqlx::query(query)
        .bind(product_id)
        .execute(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok("success".to_string())
}

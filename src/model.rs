use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: String,
    pub price: String,
}

#[derive(Debug, Serialize)]
pub struct CreateProductResponse {
    pub status_code: i32,
    pub body: String,
}

#[derive(Deserialize)]
pub struct GetProductRequest {
    pub path_parameters: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GetProductResponse {
    pub status_code: i32,
    pub body: GetProductBody,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum GetProductBody {
    String(String),
    Product(Product),
}

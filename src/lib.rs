use serde::Serialize;

pub mod utils;

#[derive(Debug, Serialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: String,
}

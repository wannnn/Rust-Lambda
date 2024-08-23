use std::collections::HashMap;

use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use rust_lambda::utils::setup_tracing;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct CreateProductRequest {
    name: String,
    description: String,
    price: String,
}

#[derive(Debug, Serialize)]
struct CreateProductResponse {
    status_code: i32,
    body: String,
}

async fn create_product_handler(
    client: &Client,
    table_name: &str,
    event: LambdaEvent<CreateProductRequest>,
) -> Result<CreateProductResponse, Error> {
    let product = event.payload;
    let product_id = Uuid::new_v4().to_string();

    // Create a new item to put into the DynamoDB table
    let mut item = HashMap::new();
    item.insert("id".to_string(), AttributeValue::S(product_id));
    item.insert("name".to_string(), AttributeValue::S(product.name.clone()));
    item.insert(
        "description".to_string(),
        AttributeValue::S(product.description.clone()),
    );
    item.insert(
        "price".to_string(),
        AttributeValue::S(product.price.clone()),
    );

    // Create a PutItem request
    let response = client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await;

    match response {
        Ok(_) => Ok(CreateProductResponse {
            status_code: 201, // 201 Created
            body: "Product created successfully".to_string(),
        }),
        Err(e) => {
            eprintln!("Failed to create item in DynamoDB: {}", e);
            Ok(CreateProductResponse {
                status_code: 500,
                body: "Failed to create product".to_string(),
            })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();

    // Get config from environment
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let table_name = "ProductTable";
    // Create the DynamoDB client
    let ddb_client = Client::new(&aws_config);

    run(service_fn(|request: LambdaEvent<CreateProductRequest>| {
        create_product_handler(&ddb_client, &table_name, request)
    }))
    .await
}

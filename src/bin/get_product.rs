use std::collections::HashMap;

use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_lambda::{utils::setup_tracing, Product};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GetProductRequest {
    path_parameters: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum GetProductBody {
    String(String),
    Product(Product),
}

#[derive(Debug, Serialize)]
struct GetProductResponse {
    status_code: i32,
    body: GetProductBody,
}

/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn get_product_handler(
    client: &Client,
    table_name: &str,
    event: LambdaEvent<GetProductRequest>,
) -> Result<GetProductResponse, Error> {
    let id = match event.payload.path_parameters {
        Some(params) => params.get("id").cloned().unwrap_or_default(),
        None => "".to_string(),
    };

    // Create a GetItem request
    let response = client
        .get_item()
        .table_name(table_name)
        .key("id", AttributeValue::S(id.clone()))
        .send()
        .await;

    match response {
        Ok(res) => {
            if let Some(item) = res.item {
                // Deserialize the item into a Product struct
                let product = Product {
                    id: item
                        .get("id")
                        .and_then(|v| v.as_s().ok())
                        .unwrap()
                        .to_string(),
                    name: item
                        .get("name")
                        .and_then(|v| v.as_s().ok())
                        .unwrap()
                        .to_string(),
                    description: item
                        .get("description")
                        .and_then(|v| v.as_s().ok())
                        .unwrap()
                        .to_string(),
                    price: item
                        .get("price")
                        .and_then(|v| v.as_s().ok())
                        .unwrap()
                        .to_string(),
                };
                Ok(GetProductResponse {
                    status_code: 200,
                    body: GetProductBody::Product(product),
                })
            } else {
                // Item not found
                Ok(GetProductResponse {
                    status_code: 404,
                    body: GetProductBody::String("Product not found".to_string()),
                })
            }
        }
        Err(e) => {
            eprintln!("Failed to get item from DynamoDB: {}", e);
            Ok(GetProductResponse {
                status_code: 500,
                body: GetProductBody::String("Failed to retrieve product".to_string()),
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

    run(service_fn(|request: LambdaEvent<GetProductRequest>| {
        get_product_handler(&ddb_client, &table_name, request)
    }))
    .await
}

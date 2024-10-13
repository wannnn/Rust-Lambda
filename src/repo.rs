use std::collections::HashMap;

use crate::model::{
    CreateProductRequest, CreateProductResponse, GetProductBody, GetProductResponse, Product,
};
use async_trait::async_trait;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_runtime::Error;
use mockall::automock;
use uuid::Uuid;

#[automock]
#[async_trait]
pub trait DynamoDbClient {
    async fn create_product(
        &self,
        payload: CreateProductRequest,
    ) -> Result<CreateProductResponse, Error>;
    async fn get_product(&self, id: String) -> Result<GetProductResponse, Error>;
}

#[async_trait]
impl DynamoDbClient for Client {
    async fn create_product(
        &self,
        payload: CreateProductRequest,
    ) -> Result<CreateProductResponse, Error> {
        let table_name = "ProductTable";

        let product_id = Uuid::new_v4().to_string();

        // Create a new item to put into the DynamoDB table
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S(product_id));
        item.insert("name".to_string(), AttributeValue::S(payload.name.clone()));
        item.insert(
            "description".to_string(),
            AttributeValue::S(payload.description.clone()),
        );
        item.insert(
            "price".to_string(),
            AttributeValue::S(payload.price.clone()),
        );

        // Create a PutItem request
        let response = self
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
    async fn get_product(&self, id: String) -> Result<GetProductResponse, Error> {
        let table_name = "ProductTable";

        // Create a GetItem request
        let response = self
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
}

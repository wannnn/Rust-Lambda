use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use rust_lambda::{
    model::{GetProductBody, GetProductRequest, GetProductResponse},
    repo::DynamoDbClient,
    utils::setup_tracing,
};

async fn get_product<T: DynamoDbClient>(
    client: &T,
    event: LambdaEvent<GetProductRequest>,
) -> Result<GetProductResponse, Error> {
    // Extract path parameter from request
    let id = match event.payload.path_parameters {
        Some(params) => params.get("id").cloned().unwrap_or_default(),
        None => {
            return Ok(GetProductResponse {
                status_code: 400,
                body: GetProductBody::String("id is required".to_string()),
            });
        }
    };

    Ok(client.get_product(id).await?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();

    // Get config from environment
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    // Create the DynamoDB client
    let ddb_client = Client::new(&aws_config);

    run(service_fn(|request: LambdaEvent<GetProductRequest>| {
        get_product(&ddb_client, request)
    }))
    .await
}

#[cfg(test)]
mod test {
    use mockall::predicate;
    use rust_lambda::{
        model::{GetProductBody, Product},
        repo::{DynamoDbClient, MockDynamoDbClient},
    };

    use crate::GetProductResponse;

    #[tokio::test]
    async fn test_get_product_success() {
        // Arrange
        let id = String::from("1");
        let body = GetProductResponse {
            status_code: 200,
            body: GetProductBody::Product(Product {
                id: "1".to_string(),
                name: "Test".to_string(),
                description: "Test".to_string(),
                price: "9.99".to_string(),
            }),
        };

        let mut mock = MockDynamoDbClient::new();

        mock.expect_get_product()
            .with(predicate::eq(id.clone()))
            .times(1)
            .returning(move |_| {
                Ok(GetProductResponse {
                    status_code: 200,
                    body: GetProductBody::Product(Product {
                        id: "1".to_string(),
                        name: "Test".to_string(),
                        description: "Test".to_string(),
                        price: "9.99".to_string(),
                    }),
                })
            });

        let response = mock.get_product(id).await.unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, body.body);
    }

    #[tokio::test]
    async fn test_get_product_not_found() {
        // Arrange
        let id = String::from("999");
        let body = GetProductResponse {
            status_code: 404,
            body: GetProductBody::String("Product not found".to_string()),
        };

        let mut mock = MockDynamoDbClient::new();

        mock.expect_get_product()
            .with(predicate::eq(id.clone()))
            .times(1)
            .returning(move |_| {
                Ok(GetProductResponse {
                    status_code: 404,
                    body: GetProductBody::String("Product not found".to_string()),
                })
            });

        // Act
        let response = mock.get_product(id).await.unwrap();

        // Assert
        assert_eq!(response.status_code, 404);
        assert_eq!(response.body, body.body);
    }

    #[tokio::test]
    async fn test_get_product_error() {
        // Arrange
        let id = String::from("123");
        let mut mock = MockDynamoDbClient::new();

        mock.expect_get_product()
            .with(predicate::eq(id.clone()))
            .times(1)
            .returning(move |_| Err("DynamoDB error".into()));

        // Act
        let result = mock.get_product(id).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "DynamoDB error");
    }
}

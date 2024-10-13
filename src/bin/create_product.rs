use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use rust_lambda::{
    model::{CreateProductRequest, CreateProductResponse},
    repo::DynamoDbClient,
    utils::setup_tracing,
};

async fn create_product<T: DynamoDbClient>(
    client: &T,
    event: LambdaEvent<CreateProductRequest>,
) -> Result<CreateProductResponse, Error> {
    Ok(client.create_product(event.payload).await?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();

    // Get config from environment
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    // Create the DynamoDB client
    let ddb_client = Client::new(&aws_config);

    run(service_fn(|request: LambdaEvent<CreateProductRequest>| {
        create_product(&ddb_client, request)
    }))
    .await
}

#[cfg(test)]
mod test {
    use mockall::predicate;
    use rust_lambda::{
        model::{CreateProductRequest, CreateProductResponse},
        repo::MockDynamoDbClient,
    };
    use lambda_runtime::LambdaEvent;
    use crate::create_product;

    #[tokio::test]
    async fn test_create_product_success() {
        // Arrange
        let request = CreateProductRequest {
            name: "Test Product".to_string(),
            description: "This is a test product".to_string(),
            price: "19.99".to_string(),
        };

        let event = LambdaEvent {
            payload: request.clone(),
            context: Default::default(),
        };

        let mut mock = MockDynamoDbClient::new();

        mock.expect_create_product()
            .with(predicate::eq(request.clone()))
            .times(1)
            .returning(move |_| {
                Ok(CreateProductResponse {
                    status_code: 201,
                    body: "Product created successfully".to_string(),
                })
            });

        // Act
        let response = create_product(&mock, event).await.unwrap();

        // Assert
        assert_eq!(response.status_code, 201);
        assert_eq!(response.body, "Product created successfully".to_string());
    }

    #[tokio::test]
    async fn test_create_product_dynamodb_error() {
        // Arrange
        let request = CreateProductRequest {
            name: "Test Product".to_string(),
            description: "This is a test product".to_string(),
            price: "19.99".to_string(),
        };

        let event = LambdaEvent {
            payload: request.clone(),
            context: Default::default(),
        };

        let mut mock = MockDynamoDbClient::new();

        mock.expect_create_product()
            .with(predicate::eq(request.clone()))
            .times(1)
            .returning(move |_| {
                Ok(CreateProductResponse {
                    status_code: 500,
                    body: "Failed to create product".to_string(),
                })
            });

        // Act
        let response = create_product(&mock, event).await.unwrap();

        // Assert
        assert_eq!(response.status_code, 500);
        assert_eq!(response.body, "Failed to create product".to_string());
    }
}

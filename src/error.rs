use aws_sdk_dynamodb::{
    error::SdkError,
    operation::{
        create_table::CreateTableError, delete_table::DeleteTableError,
        describe_table::DescribeTableError,
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DynamoToolsError {
    #[error("Failed to read configuration file '{0}': {1}")]
    ConfigRead(String, #[source] std::io::Error),

    #[error("Failed to parse configuration file '{0}': {1}")]
    ConfigParse(String, #[source] serde_yaml::Error),

    #[error("Failed to build AWS SDK config: {0}")]
    AwsSdkConfig(#[from] aws_sdk_dynamodb::error::BuildError),

    #[error("Missing expected field in configuration or SDK response: {0}")]
    MissingField(String),
    #[error("AWS DynamoDB SDK error: {0}")]
    DynamoDbSdk(#[from] aws_sdk_dynamodb::Error),

    #[error("AWS SDK error during table creation: {0}")]
    TableCreation(#[from] SdkError<CreateTableError>),

    #[error("AWS SDK error during table deletion: {0}")]
    TableDeletion(#[from] SdkError<DeleteTableError>),

    #[error("AWS SDK error during table description: {0}")]
    TableDescribe(#[from] SdkError<DescribeTableError>),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, DynamoToolsError>;

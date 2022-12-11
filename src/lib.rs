mod config;
#[cfg(feature = "local")]
mod local_client;

use aws_sdk_dynamodb::Client;
pub use config::TableConfig;
#[cfg(feature = "local")]
pub use local_client::{get_client, local_client, LocalClient};

/// DynamoClient is a trait that provides a DynamoDB client and table name
pub trait DynamoClient {
    /// return the DynamoDB client
    fn client(&self) -> &Client;
    /// for non-local client, this method returns None
    fn table_name(&self) -> Option<&str> {
        None
    }
}

impl DynamoClient for Client {
    fn client(&self) -> &Client {
        self
    }
}

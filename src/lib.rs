mod config;
#[cfg(feature = "connector")]
mod connector;

pub use config::{TableConfig, TableInfo};
#[cfg(feature = "connector")]
pub use connector::DynamodbConnector;

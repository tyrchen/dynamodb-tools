#![doc = include_str!("../README.md")]

mod config;
#[cfg(feature = "connector")]
mod connector;
pub mod error;

// Make config structs/enums public for test construction
pub use config::{AttrType, TableAttr, TableConfig, TableInfo};
#[cfg(feature = "connector")]
pub use connector::DynamodbConnector;
pub use error::{DynamoToolsError, Result};

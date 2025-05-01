use crate::TableConfig;
use crate::error::{DynamoToolsError, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::config::Credentials;
use aws_sdk_dynamodb::{Client, operation::create_table::CreateTableInput};
use std::path::Path;
#[cfg(feature = "test_utils")]
use tokio::runtime::Runtime;

/// Provides a connection to DynamoDB, potentially managing a test table lifecycle.
///
/// This struct encapsulates an AWS DynamoDB client (`aws_sdk_dynamodb::Client`).
/// If configured with table `info` and a local `endpoint` in [`TableConfig`],
/// it will create a uniquely named table upon construction.
///
/// If the `test_utils` feature is enabled and `delete_on_exit` is true in the
/// configuration, the created table will be automatically deleted when this
/// connector is dropped.
#[derive(Debug, Clone)]
pub struct DynamodbConnector {
    // Keep client private, expose via method
    client: Option<Client>,
    // Keep table_name private, expose via method
    table_name: String,
    // Test utility flag
    #[cfg(feature = "test_utils")]
    delete_on_exit: bool,
}

impl DynamodbConnector {
    /// Creates a new connector by loading configuration from a YAML file.
    ///
    /// See [`TableConfig::load_from_file`] and [`DynamodbConnector::try_new`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if loading the config file fails or if creating the connector fails
    /// (e.g., table creation fails, AWS configuration error).
    pub async fn load(config_path: impl AsRef<Path>) -> Result<Self> {
        let config = TableConfig::load_from_file(config_path)?;
        DynamodbConnector::try_new(config).await
    }

    /// Returns a reference to the underlying `aws_sdk_dynamodb::Client`.
    ///
    /// # Errors
    ///
    /// Returns `Err` ([`DynamoToolsError::Internal`]) if the client has already been
    /// taken (e.g., after `Drop` has started).
    pub fn client(&self) -> Result<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| DynamoToolsError::Internal("Client accessed after Drop".to_string()))
    }

    /// Returns the name of the table associated with this connector.
    ///
    /// If table creation was configured, this will be the uniquely generated name
    /// (e.g., `base_name-unique_id`). Otherwise, it's the `table_name` from the config.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dynamodb_tools::{DynamodbConnector, TableConfig, Result};
    /// # use tokio::runtime::Runtime;
    /// #
    /// # fn main() -> Result<()> {
    /// #     let rt = Runtime::new().unwrap();
    /// #     rt.block_on(async {
    /// // Load config that doesn't define table schema (`info: None`)
    /// let config = TableConfig::load_from_file("fixtures/prod.yml")?;
    /// let connector = DynamodbConnector::try_new(config).await?;
    ///
    /// // table_name() returns the name directly from the config
    /// assert_eq!(connector.table_name(), "users");
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn table_name(&self) -> &str {
        self.table_name.as_str()
    }

    /// Returns whether the connector is configured to delete the table on drop.
    ///
    /// This requires the `test_utils` feature to be enabled.
    ///
    /// ```rust
    /// # use dynamodb_tools::{DynamodbConnector, TableConfig, Result};
    /// # use tokio::runtime::Runtime;
    /// #
    /// # fn main() -> Result<()> {
    /// #     let rt = Runtime::new().unwrap();
    /// #     rt.block_on(async {
    /// // Load config with delete_on_exit: true and an endpoint
    /// let config = TableConfig::load_from_file("fixtures/dev.yml")?;
    /// let connector = DynamodbConnector::try_new(config).await?;
    ///
    /// // delete_on_exit() returns the configured value
    /// #[cfg(feature = "test_utils")] // This assertion only runs if the feature is enabled
    /// assert!(connector.delete_on_exit());
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    #[cfg(feature = "test_utils")]
    pub fn delete_on_exit(&self) -> bool {
        self.delete_on_exit
    }

    /// Creates a new connector based on the provided [`TableConfig`].
    ///
    /// - Sets up AWS SDK configuration (using environment defaults or test credentials for local endpoints).
    /// - Creates a `aws_sdk_dynamodb::Client`.
    /// - If `config.info` is `Some`, attempts to create a DynamoDB table with a unique name
    ///   derived from `config.table_name` and `config.info` schema.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - Reading AWS configuration fails.
    /// - Building the DynamoDB client configuration fails ([`DynamoToolsError::AwsSdkConfig`]).
    /// - Table schema conversion fails.
    /// - Required fields are missing ([`DynamoToolsError::MissingField`]).
    /// - The `CreateTable` API call fails ([`DynamoToolsError::TableCreation`]).
    pub async fn try_new(config: TableConfig) -> Result<Self> {
        let endpoint = config.endpoint.clone();
        #[cfg(feature = "test_utils")]
        let delete_on_exit = if endpoint.is_some() {
            config.delete_on_exit
        } else {
            false
        };

        let base_sdk_config_builder = aws_config::defaults(BehaviorVersion::latest()).region(
            RegionProviderChain::first_try(Region::new(config.region.clone()))
                .or_default_provider(),
        );

        let loaded_sdk_config = base_sdk_config_builder.load().await;

        let builder = aws_sdk_dynamodb::config::Builder::from(&loaded_sdk_config);
        let dynamodb_config = if let Some(url) = endpoint.as_ref() {
            builder
                .endpoint_url(url)
                .credentials_provider(Credentials::for_tests())
                .build()
        } else {
            builder.build()
        };

        let client = Client::from_conf(dynamodb_config);

        let table_name = if let Some(info) = config.info {
            let mut input = CreateTableInput::try_from(info)?;
            let base_table_name = input.table_name.clone().ok_or_else(|| {
                DynamoToolsError::MissingField("Table name missing in TableInfo".to_string())
            })?;

            let unique_table_name = format!("{}-{}", base_table_name, xid::new());
            input.table_name = Some(unique_table_name.clone());

            let create_table_builder = client
                .create_table()
                .table_name(&unique_table_name)
                .set_key_schema(input.key_schema)
                .set_attribute_definitions(input.attribute_definitions)
                .set_global_secondary_indexes(input.global_secondary_indexes)
                .set_local_secondary_indexes(input.local_secondary_indexes);

            let create_table_builder = match input.provisioned_throughput {
                Some(pt) => create_table_builder.provisioned_throughput(pt),
                None => create_table_builder.billing_mode(input.billing_mode.ok_or_else(|| {
                    DynamoToolsError::MissingField(
                        "Billing mode must exist if provisioned throughput is not set".to_string(),
                    )
                })?),
            };

            create_table_builder.send().await?;
            unique_table_name
        } else {
            config.table_name
        };

        Ok(Self {
            client: Some(client),
            table_name,
            #[cfg(feature = "test_utils")]
            delete_on_exit,
        })
    }
}

/// Best-effort table cleanup on drop (requires `test_utils` feature).
///
/// If `delete_on_exit` was true and an endpoint was configured,
/// attempts to delete the table in a background thread.
/// Logs errors using `eprintln!`. Use explicit cleanup methods if reliable
/// cleanup is required within an async context.
#[cfg(feature = "test_utils")]
impl Drop for DynamodbConnector {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let table_name = self.table_name.clone();
            if !self.delete_on_exit {
                return;
            }
            std::thread::spawn(move || {
                let rt = match Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        eprintln!(
                            "[ERROR] Failed to create Tokio runtime for table deletion: {}",
                            e
                        );
                        return;
                    }
                };

                rt.block_on(async move {
                    match client.delete_table().table_name(&table_name).send().await {
                        Ok(_) => println!("[INFO] Deleted table: {}", table_name),
                        Err(e) => {
                            eprintln!("[ERROR] Failed to delete table '{}': {}", table_name, e)
                        }
                    }
                });
            });
        }
    }
}

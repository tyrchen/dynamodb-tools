use crate::TableConfig;
use crate::error::{DynamoToolsError, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::config::Credentials;
use aws_sdk_dynamodb::{Client, operation::create_table::CreateTableInput};
use std::{collections::HashMap, path::Path};
#[cfg(feature = "test_utils")]
use tokio::runtime::Runtime;

/// Provides a connection to DynamoDB, potentially managing test table lifecycles.
///
/// This struct encapsulates an AWS DynamoDB client (`aws_sdk_dynamodb::Client`).
/// If configured with table definitions and a local `endpoint` in [`TableConfig`],
/// it will create uniquely named tables upon construction.
///
/// If the `test_utils` feature is enabled and `delete_on_exit` is true in the
/// configuration, the created tables will be automatically deleted when this
/// connector is dropped.
#[derive(Debug)]
pub struct DynamodbConnector {
    client: Option<Client>,
    // Map base table name to actual unique table name created
    created_tables: HashMap<String, String>,
    // Keep track of the original config for Drop
    #[cfg(feature = "test_utils")]
    config: TableConfig,
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
            .ok_or_else(|| DynamoToolsError::Internal("Client instance is missing".to_string()))
    }

    /// Returns the unique name of a table created by this connector, given its base name.
    ///
    /// The `base_name` corresponds to the `table_name` field within [`TableInfo`]
    /// in the configuration.
    pub fn get_created_table_name(&self, base_name: &str) -> Option<&str> {
        self.created_tables.get(base_name).map(|s| s.as_str())
    }

    /// Returns a map of all tables created by this connector.
    /// Keys are the base names from the config, values are the unique created names.
    pub fn get_all_created_table_names(&self) -> &HashMap<String, String> {
        &self.created_tables
    }

    /// Creates a new connector based on the provided [`TableConfig`].
    ///
    /// - Sets up AWS SDK configuration.
    /// - Creates a `aws_sdk_dynamodb::Client`.
    /// - Iterates through `config.tables`. For each `TableInfo`:
    ///   - Attempts to create a DynamoDB table with a unique name derived from `TableInfo.table_name`.
    ///   - Stores the mapping from the base name to the unique name.
    ///
    /// # Errors
    ///
    /// Returns `Err` if AWS config fails, client creation fails, or any table creation fails.
    pub async fn try_new(config: TableConfig) -> Result<Self> {
        let endpoint = config.endpoint.clone();
        // Store config for Drop
        #[cfg(feature = "test_utils")]
        let connector_config = config.clone();

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

        let mut created_tables = HashMap::new();

        // Iterate through table definitions and create each one
        for table_info in config.tables {
            let base_table_name = table_info.table_name.clone(); // Keep original name for map key
            let mut input = CreateTableInput::try_from(table_info)?;

            // Generate unique name
            let unique_table_name = format!("{}-{}", base_table_name, xid::new());
            // Overwrite table name in the input with the unique one
            input.table_name = Some(unique_table_name.clone());

            // Build the CreateTable request (logic adapted from previous version)
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
                    DynamoToolsError::MissingField(format!(
                        "Billing mode missing for table '{}' with no throughput",
                        base_table_name
                    ))
                })?),
            };

            // Send the request
            create_table_builder
                .send()
                .await
                .map_err(DynamoToolsError::TableCreation)?; // Propagate SDK errors, wrapped in our type

            // Store the mapping
            created_tables.insert(base_table_name, unique_table_name);
        }

        Ok(Self {
            client: Some(client),
            created_tables,
            #[cfg(feature = "test_utils")]
            config: connector_config,
        })
    }
}

/// Best-effort table cleanup on drop (requires `test_utils` feature).
///
/// If `delete_on_exit` was true and an endpoint was configured,
/// attempts to delete all tables created by this connector in background threads.
#[cfg(feature = "test_utils")]
impl Drop for DynamodbConnector {
    fn drop(&mut self) {
        // Check config before taking client
        if !self.config.delete_on_exit || self.config.endpoint.is_none() {
            println!(
                "[INFO] Skipping delete on drop (delete_on_exit: {}, endpoint: {:?})",
                self.config.delete_on_exit, self.config.endpoint
            );
            return;
        }

        if let Some(client) = self.client.take() {
            // Clone map and config needed for threads
            let tables_to_delete = self.created_tables.clone();
            println!(
                "[INFO] Drop: Attempting to delete tables: {:?}",
                tables_to_delete.values()
            );

            for (_base_name, unique_name) in tables_to_delete {
                let client_clone = client.clone(); // Clone client for each thread
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
                        match client_clone
                            .delete_table()
                            .table_name(&unique_name)
                            .send()
                            .await
                        {
                            Ok(_) => println!("[INFO] Deleted table: {}", unique_name),
                            Err(e) => {
                                eprintln!("[ERROR] Failed to delete table '{}': {}", unique_name, e)
                            }
                        }
                    });
                });
            }
        }
    }
}

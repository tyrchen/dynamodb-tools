use crate::TableConfig;
use crate::error::{DynamoToolsError, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::config::Credentials;
use aws_sdk_dynamodb::{Client, operation::create_table::CreateTableInput};
use std::path::Path;
#[cfg(feature = "test_utils")]
use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct DynamodbConnector {
    client: Option<Client>,
    table_name: String,
    #[cfg(feature = "test_utils")]
    delete_on_exit: bool,
}

impl DynamodbConnector {
    /// Load dynamodb connector from configuration file
    pub async fn load(config_path: impl AsRef<Path>) -> Result<Self> {
        let config = TableConfig::load_from_file(config_path)?;
        DynamodbConnector::try_new(config).await
    }

    pub fn client(&self) -> Result<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| DynamoToolsError::Internal("Client accessed after Drop".to_string()))
    }

    pub fn table_name(&self) -> &str {
        self.table_name.as_str()
    }

    #[cfg(feature = "test_utils")]
    pub fn delete_on_exit(&self) -> bool {
        self.delete_on_exit
    }

    /// create a new local client
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

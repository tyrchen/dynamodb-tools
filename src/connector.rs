use crate::TableConfig;
use crate::error::{DynamoToolsError, Result};
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{Client, operation::create_table::CreateTableInput};
use std::path::Path;
#[cfg(test)]
use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct DynamodbConnector {
    client: Option<Client>,
    table_name: String,
    #[cfg(test)]
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

    /// create a new local client
    pub async fn try_new(table_config: TableConfig) -> Result<Self> {
        let local_endpoint = table_config.local_endpoint.clone();
        #[cfg(test)]
        let delete_on_exit = if local_endpoint.is_some() {
            table_config.delete_on_exit
        } else {
            false
        };
        let config = aws_config::load_from_env().await;

        let sdk_config_builder = aws_sdk_dynamodb::Config::builder()
            .region(config.region().cloned())
            .behavior_version(
                config
                    .behavior_version()
                    .unwrap_or_else(BehaviorVersion::latest),
            )
            .credentials_provider(
                config
                    .credentials_provider()
                    .ok_or_else(|| {
                        DynamoToolsError::MissingField(
                            "AWS credentials provider not found".to_string(),
                        )
                    })?
                    .clone(),
            );

        let sdk_config_builder = if let Some(url) = local_endpoint.as_ref() {
            sdk_config_builder.endpoint_url(url)
        } else {
            sdk_config_builder
        };
        let sdk_config = sdk_config_builder.build();
        let client = Client::from_conf(sdk_config);

        let table_name = if let Some(info) = table_config.info {
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
            table_config.table_name
        };

        Ok(Self {
            client: Some(client),
            table_name,
            #[cfg(test)]
            delete_on_exit,
        })
    }
}

#[cfg(test)]
impl Drop for DynamodbConnector {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let table_name = self.table_name.clone();
            #[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[tokio::test]
    async fn dev_config_should_work() -> Result<()> {
        let config = TableConfig::load_from_file("fixtures/dev.yml")?;
        let connector = DynamodbConnector::try_new(config).await?;
        let table_name = connector.table_name().to_string();
        let resp = connector
            .client()?
            .describe_table()
            .table_name(&table_name)
            .send()
            .await?;
        assert_eq!(resp.table.and_then(|v| v.table_name).unwrap(), table_name);
        Ok(())
    }

    #[tokio::test]
    async fn prod_config_should_work() -> Result<()> {
        let config = TableConfig::load_from_file("fixtures/prod.yml")?;
        let connector = DynamodbConnector::try_new(config).await?;
        let table_name = connector.table_name();
        assert_eq!(table_name, "users");
        Ok(())
    }
}

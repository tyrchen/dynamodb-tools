use crate::TableConfig;
use anyhow::Result;
use aws_sdk_dynamodb::{input::CreateTableInput, Client, Endpoint};
use std::{path::Path, thread};
use tokio::runtime::Runtime;

pub struct DynamodbConnector {
    client: Option<Client>,
    table_name: String,
    delete_on_exit: bool,
}

impl DynamodbConnector {
    /// Load dynamodb connector from configuration file
    pub async fn load(config: impl AsRef<Path>) -> Result<Self> {
        let config = TableConfig::load_from_file(config)?;
        DynamodbConnector::try_new(config).await
    }

    pub fn client(&self) -> &Client {
        self.client.as_ref().expect("client should exists")
    }

    pub fn table_name(&self) -> &str {
        self.table_name.as_str()
    }

    /// create a new local client
    pub async fn try_new(table_config: TableConfig) -> Result<Self> {
        let local_endpoint = table_config.local_endpoint.clone();
        let delete_on_exit = if local_endpoint.is_some() {
            table_config.delete_on_exit
        } else {
            false
        };
        let config = aws_config::load_from_env().await;

        let config = aws_sdk_dynamodb::Config::builder()
            .region(config.region().cloned())
            .credentials_provider(
                config
                    .credentials_provider()
                    .expect("cred should exists")
                    .clone(),
            );

        let config = if let Some(url) = local_endpoint.as_ref() {
            config.endpoint_resolver(Endpoint::immutable(url.parse()?))
        } else {
            config
        };
        let client = Client::from_conf(config.build());

        let table_name = if let Some(info) = table_config.info {
            let mut input = CreateTableInput::try_from(info)?;
            let table_name = format!("{}-{}", input.table_name.unwrap(), xid::new());
            input.table_name = Some(table_name.clone());

            client
                .create_table()
                .table_name(&table_name)
                .set_key_schema(input.key_schema)
                .set_attribute_definitions(input.attribute_definitions)
                .set_global_secondary_indexes(input.global_secondary_indexes)
                .set_local_secondary_indexes(input.local_secondary_indexes)
                .provisioned_throughput(input.provisioned_throughput.take().unwrap())
                .send()
                .await?;
            table_name
        } else {
            table_config.table_name
        };

        Ok(Self {
            client: Some(client),
            table_name,
            delete_on_exit,
        })
    }
}

impl Drop for DynamodbConnector {
    fn drop(&mut self) {
        let client = self.client.take().expect("client");
        let table_name = self.table_name.clone();
        if !self.delete_on_exit {
            return;
        }
        thread::spawn(move || {
            let rt = Runtime::new().expect("runtime");
            rt.block_on(async move {
                if let Err(e) = client.delete_table().table_name(&table_name).send().await {
                    println!("failed to delete table: {:?}", e);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dev_config_should_work() {
        let config = TableConfig::load_from_file("fixtures/dev.yml").unwrap();
        let connector = DynamodbConnector::try_new(config).await.unwrap();
        let table_name = connector.table_name();
        let resp = connector
            .client()
            .describe_table()
            .table_name(table_name)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.table.and_then(|v| v.table_name).unwrap(), table_name);
    }

    #[tokio::test]
    async fn prod_config_should_work() {
        let config = TableConfig::load_from_file("fixtures/prod.yml").unwrap();
        let connector = DynamodbConnector::try_new(config).await.unwrap();
        let table_name = connector.table_name();
        assert_eq!(table_name, "users");
        assert!(!connector.delete_on_exit);
    }
}

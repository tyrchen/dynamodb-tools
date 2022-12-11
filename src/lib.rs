mod config;

use anyhow::Result;
use aws_sdk_dynamodb::{input::CreateTableInput, Client, Endpoint};
use config::TableConfig;
use std::thread;
use tokio::runtime::Runtime;

pub struct LocalClient {
    client: Option<Client>,
    table_name: String,
}

impl LocalClient {
    /// create a new local client
    pub async fn try_new(port: u16, table_config: TableConfig) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let mut input = CreateTableInput::try_from(table_config)?;
        let table_name = format!("{}-{}", input.table_name.unwrap(), xid::new());
        input.table_name = Some(table_name.clone());
        let dynamodb_local_config = aws_sdk_dynamodb::Config::builder()
            .region(config.region().cloned())
            .credentials_provider(
                config
                    .credentials_provider()
                    .expect("cred should exists")
                    .clone(),
            )
            .endpoint_resolver(Endpoint::immutable(
                format!("http://localhost:{port}").parse().expect("valid"),
            ))
            .build();
        let client = Client::from_conf(dynamodb_local_config);

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
        Ok(Self {
            client: Some(client),
            table_name,
        })
    }

    /// retrieve the client reference and table name
    pub fn inner(&self) -> (&Client, &str) {
        (
            self.client.as_ref().expect("client"),
            self.table_name.as_str(),
        )
    }
}

impl Drop for LocalClient {
    fn drop(&mut self) {
        let client = self.client.take().expect("client");
        let table_name = self.table_name.clone();
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
    async fn test_local_client() {
        let config = TableConfig::load_from_file("fixtures/config.yml").unwrap();
        let client = LocalClient::try_new(8000, config).await.unwrap();
        let (client, table_name) = client.inner();
        let resp = client
            .describe_table()
            .table_name(table_name)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.table.and_then(|v| v.table_name).unwrap(), table_name);
    }
}

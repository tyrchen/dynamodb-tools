# DynamoDB tools

This crate is previously called [dynamodb-tester](https://crates.io/crates/dynamodb-tester), but I decided to rename it to dynamodb-tools, because it is not only for testing.

As AWS provided [DynamoDB local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.html), we could leverage it in the development & test environment. However, managing the dynamodb client and tables is tedious, we need to clean that up at the end of every test to not pollute other tests. This crate will help you to create tables with unique names and then tear them down after test ends (by using Drop trait if you ask).

## Usage

First you need to download and run dynamodb local yourself. For example, I unzipped it in ~/bin/dynamodb_local_latest, so I can start it like this:

```bash
$ java -Djava.library.path=~/bin/dynamodb_local_latest/DynamoDBLocal_lib -jar ~/bin/dynamodb_local_latest/DynamoDBLocal.jar -inMemory -sharedDb
```

Feel free to make it a service so that it starts automatically when system starts.

In your test code, you could use it like this:

```rust
// first, create the LocalClient
use dynamodb_tools::{DynamodbConnector, TableConfig};
use anyhow::Result;
use serde_dynamo::to_item;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
  pk: String,
  sk: String,
  gsi1pk: String,
  gsi1sk: String,
}

// Add feature gate comment
// Requires feature `test_utils` if using delete_on_exit: true
#[tokio::main]
async fn main() -> Result<()> {
  let config = TableConfig::load_from_file("fixtures/dev.yml")?;
  let connector = DynamodbConnector::try_new(config).await?;

  // Get the actual created table name using the base name from config
  let actual_table_name = connector.get_created_table_name("users")
                           .expect("Table 'users' should have been created");

  let item = to_item(Item {
    pk: "1".to_string(),
    sk: "1".to_string(),
    gsi1pk: "1".to_string(),
    gsi1sk: "1".to_string(),
  })?;

  let ret = connector.client()?
      .put_item()
      .table_name(actual_table_name) // Use the retrieved unique name
      .set_item(Some(item))
      .send()
      .await?;
  Ok(())
}
```

If you want to integrate it with github action, you could use [this action](https://github.com/rrainn/dynamodb-action):

```yaml
- name: Setup DynamoDB Local
  uses: rrainn/dynamodb-action@v2.0.1
  with:
  port: 8000
  cors: '*'
```

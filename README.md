# DynamoDB tools

This crate is previously called [dynamodb-tester](https://crates.io/crates/dynamodb-tester), but I decided to rename it to dynamodb-tools, because it is not only for testing.

As AWS provided [DynamoDB local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.html), we could leverage it in the development & test environment. However, managing the dynamodb client and tables is tedious, we need to clean that up at the end of every test to not pollute other tests. This crate will help you to:

*   Define schemas for one or more tables in a YAML configuration file.
*   Optionally specify a JSON file to seed each table with initial data.
*   Create uniquely named tables based on your schemas when connecting (ideal for tests).
*   Optionally tear down the created tables automatically when the connector goes out of scope (using the `test_utils` feature).

## Usage

First you need to download and run dynamodb local yourself. For example, I unzipped it in ~/bin/dynamodb_local_latest, so I can start it like this:

```bash
$ java -Djava.library.path=~/bin/dynamodb_local_latest/DynamoDBLocal_lib -jar ~/bin/dynamodb_local_latest/DynamoDBLocal.jar -inMemory -sharedDb
```

Feel free to make it a service so that it starts automatically when system starts.

### Example Configuration (`config.yml`)

```yaml
# Global settings
region: us-east-1
endpoint: http://localhost:8000 # Target DynamoDB Local
delete_on_exit: true          # Requires 'test_utils' feature

# List of tables to manage
tables:
  - table_name: users        # Base name for the 'users' table
    pk:
      name: user_id
      type: S
    sk:
      name: resource_type # e.g., "profile", "order"
      type: S
    seed_data_file: fixtures/seed_users.json # Optional seeding
    # ... other schema details like attrs, gsis, lsis ...

  - table_name: products     # Base name for the 'products' table
    pk:
      name: product_id
      type: S
    # ... other schema details ...
```

### Example Seed Data (`fixtures/seed_users.json`)

```json
[
  {
    "user_id": "user_1",
    "resource_type": "profile",
    "name": "Alice"
  },
  {
    "user_id": "user_2",
    "resource_type": "profile",
    "name": "Bob"
  }
]
```

### Example Usage

```rust,ignore
// Assuming the config files and seed files above exist

use dynamodb_tools::{DynamodbConnector, TableConfig};
// ... other necessary imports like serde, tokio, anyhow ...

// Add feature gate comment
// Requires feature `test_utils` if using delete_on_exit: true
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config = TableConfig::load_from_file("config.yml")?;
  let connector = DynamodbConnector::try_new(config).await?;

  // Get the actual created table name using the base name from config
  let users_table = connector.get_created_table_name("users")
                           .expect("Table 'users' should have been created");
  let products_table = connector.get_created_table_name("products")
                           .expect("Table 'products' should have been created");

  println!("Created users table: {}", users_table);
  println!("Created products table: {}", products_table);

  // Get a seeded user item
  let get_resp = connector.client()?
      .get_item()
      .table_name(users_table)
      .key("user_id", aws_sdk_dynamodb::types::AttributeValue::S("user_1".to_string()))
      .key("resource_type", aws_sdk_dynamodb::types::AttributeValue::S("profile".to_string()))
      .send()
      .await?;

  if let Some(item) = get_resp.item {
      println!("Got item: {:?}", item);
      // Convert item back to a struct using serde_dynamo::from_item if needed
  } else {
      println!("Item not found!");
  }

  // ... interact with products_table ...

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

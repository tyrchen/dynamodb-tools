// tests/connector_integration_test.rs
#![cfg(feature = "test_utils")]
use aws_sdk_dynamodb::types::AttributeValue;
#[cfg(feature = "test_utils")]
use dynamodb_tools::{AttrType, DynamoToolsError, TableInfo};
use dynamodb_tools::{DynamodbConnector, Result, TableConfig};
use std::collections::HashMap;

// Note: Assumes DynamoDB Local is running at http://localhost:8000

#[cfg(feature = "test_utils")]
#[tokio::test]
async fn dev_config_should_create_and_describe_table() -> Result<()> {
    // dev.yml now defines one table with base name "users"
    let config = TableConfig::load_from_file("fixtures/dev.yml")?;
    let connector = DynamodbConnector::try_new(config).await?;

    // Get the unique name using the base name
    let unique_table_name = connector
        .get_created_table_name("users")
        .expect("Table 'users' should have been created");

    assert!(unique_table_name.starts_with("users-"));

    // Basic check: describe the created table
    let resp = connector
        .client()?
        .describe_table()
        .table_name(unique_table_name)
        .send()
        .await?;

    assert_eq!(resp.table.unwrap().table_name.unwrap(), unique_table_name);

    // Access delete_on_exit via config stored in connector (or directly if made public)
    // assert!(connector.config.delete_on_exit); // Assumes config field is accessible

    Ok(())
}

#[tokio::test]
async fn prod_config_should_return_empty_map_without_creating() -> Result<()> {
    // prod.yml has no `tables` list (or it's empty)
    let config = TableConfig::load_from_file("fixtures/prod.yml")?;
    let connector = DynamodbConnector::try_new(config).await?;

    // Check that no tables were created
    assert!(connector.get_all_created_table_names().is_empty());

    Ok(())
}

#[cfg(feature = "test_utils")]
#[tokio::test]
async fn simple_pk_table_should_allow_put() -> Result<()> {
    // Define TableInfo inline for a simple table
    let table_info = TableInfo {
        table_name: "simple_pk_test".to_string(), // Base name
        pk: dynamodb_tools::TableAttr {
            name: "id".to_string(),
            attr_type: AttrType::S,
        },
        sk: None,
        attrs: vec![],
        gsis: vec![],
        lsis: vec![],
        throughput: None,
        seed_data_file: None,
    };

    // Create TableConfig with a list containing the single table info
    let config = TableConfig {
        region: "us-east-1".to_string(),
        endpoint: Some("http://localhost:8000".to_string()),
        delete_on_exit: true,
        tables: vec![table_info],
    };

    let connector = DynamodbConnector::try_new(config).await?;

    // Get the unique name using the base name
    let unique_table_name = connector
        .get_created_table_name("simple_pk_test")
        .expect("Table 'simple_pk_test' should have been created");

    assert!(unique_table_name.starts_with("simple_pk_test-"));

    // Prepare an item to put
    let item_id = "test-item-1";
    let item = HashMap::from([(
        "id".to_string(),
        aws_sdk_dynamodb::types::AttributeValue::S(item_id.to_string()),
    )]);

    // Perform PutItem
    let put_resp = connector
        .client()?
        .put_item()
        .table_name(unique_table_name)
        .set_item(Some(item))
        .send()
        .await;

    assert!(put_resp.is_ok(), "PutItem failed: {:?}", put_resp.err());

    Ok(())
}

#[cfg(feature = "test_utils")]
#[tokio::test]
async fn multi_table_config_should_create_all_tables() -> Result<()> {
    let config = TableConfig::load_from_file("fixtures/multi_table.yml")?;
    let connector = DynamodbConnector::try_new(config).await?;

    // Check that two tables were created
    assert_eq!(connector.get_all_created_table_names().len(), 2);

    // Get table 1
    let table1_name = connector
        .get_created_table_name("multi_table_1")
        .expect("Table 'multi_table_1' should exist");
    assert!(table1_name.starts_with("multi_table_1-"));

    // Describe table 1
    let resp1 = connector
        .client()?
        .describe_table()
        .table_name(table1_name)
        .send()
        .await?;
    assert_eq!(resp1.table.unwrap().table_name.unwrap(), table1_name);

    // Get table 2
    let table2_name = connector
        .get_created_table_name("multi_table_2")
        .expect("Table 'multi_table_2' should exist");
    assert!(table2_name.starts_with("multi_table_2-"));

    // Describe table 2
    let resp2 = connector
        .client()?
        .describe_table()
        .table_name(table2_name)
        .send()
        .await?;
    assert_eq!(resp2.table.unwrap().table_name.unwrap(), table2_name);

    Ok(())
}

#[cfg(feature = "test_utils")]
#[tokio::test]
async fn dev_config_should_seed_data() -> Result<()> {
    // dev.yml configures the 'users' table with seed_data_file: fixtures/seed_users.json
    let config = TableConfig::load_from_file("fixtures/dev.yml")?;
    let connector = DynamodbConnector::try_new(config).await?;

    let table_name = connector.get_created_table_name("users").unwrap();

    // Attempt to get one of the seeded items
    let pk_val = "user_1";
    let sk_val = "profile";

    let resp = connector
        .client()?
        .get_item()
        .table_name(table_name)
        .key("pk", AttributeValue::S(pk_val.to_string()))
        .key("sk", AttributeValue::S(sk_val.to_string()))
        .send()
        .await
        .map_err(|e| DynamoToolsError::Internal(format!("GetItem failed: {}", e)))?;

    // Check if the item was found and has expected data
    match resp.item() {
        Some(item) => {
            assert_eq!(item.get("pk"), Some(&AttributeValue::S(pk_val.to_string())));
            assert_eq!(item.get("sk"), Some(&AttributeValue::S(sk_val.to_string())));
            assert_eq!(
                item.get("name"),
                Some(&AttributeValue::S("Alice".to_string()))
            );
            assert_eq!(
                item.get("email"),
                Some(&AttributeValue::S("alice@example.com".to_string()))
            );
        }
        None => {
            panic!(
                "Seeded item user_1/profile not found in table {}",
                table_name
            );
        }
    }

    Ok(())
}

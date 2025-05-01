// tests/connector_integration_test.rs

// Use the crate itself
use dynamodb_tools::{DynamodbConnector, TableConfig};
// Import the crate's error type if needed for function signatures
use dynamodb_tools::error::Result;

// Note: Assumes DynamoDB Local is running at http://localhost:8000

#[cfg(feature = "test_utils")]
#[tokio::test]
async fn dev_config_should_create_and_describe_table() -> Result<()> {
    let config = TableConfig::load_from_file("fixtures/dev.yml")?;
    let connector = DynamodbConnector::try_new(config).await?;
    let table_name = connector.table_name().to_string();

    // Basic check: describe the created table
    let resp = connector
        .client()?
        .describe_table()
        .table_name(&table_name)
        .send()
        .await?;

    // Check if the description matches the expected table name
    // The generated name includes a unique ID
    assert!(table_name.starts_with("users-"));
    assert_eq!(resp.table.unwrap().table_name.unwrap(), table_name);

    // Assertions related to the connector state (if needed and accessible)
    // Note: delete_on_exit is #[cfg(test)] gated in the struct definition
    // We can't directly assert its value here unless we make it public or add accessors.
    // This test implicitly relies on Drop working due to delete_on_exit: true in dev.yml
    assert!(connector.delete_on_exit());

    Ok(())
}

#[tokio::test]
async fn prod_config_should_return_correct_name_without_creating() -> Result<()> {
    let config = TableConfig::load_from_file("fixtures/prod.yml")?;
    // prod.yml has no `info` block, so try_new should not create a table
    let connector = DynamodbConnector::try_new(config).await?;
    let table_name = connector.table_name();

    // Check the table name directly from the config
    assert_eq!(table_name, "users");

    Ok(())
}

#[cfg(feature = "test_utils")]
#[tokio::test]
async fn simple_pk_table_should_allow_put() -> Result<()> {
    // Define TableInfo inline for a simple table
    let table_info = dynamodb_tools::TableInfo {
        table_name: "simple_pk_test".to_string(),
        pk: dynamodb_tools::TableAttr {
            name: "id".to_string(),
            attr_type: dynamodb_tools::AttrType::S,
        },
        sk: None,
        attrs: vec![], // PK automatically included
        gsis: vec![],
        lsis: vec![],
        throughput: None, // Use default PayPerRequest
    };

    // Create TableConfig using the inline TableInfo
    let config = TableConfig {
        table_name: "simple_pk_test".to_string(), // Base name, will get unique suffix
        endpoint: Some("http://localhost:8000".to_string()),
        region: "us-east-1".to_string(), // Example region
        delete_on_exit: true,
        info: Some(table_info),
    };

    let connector = DynamodbConnector::try_new(config).await?;
    let table_name = connector.table_name().to_string();

    assert!(table_name.starts_with("simple_pk_test-"));

    // Prepare an item to put
    let item_id = "test-item-1";
    let item = std::collections::HashMap::from([(
        "id".to_string(),
        aws_sdk_dynamodb::types::AttributeValue::S(item_id.to_string()),
    )]);

    // Perform PutItem
    let put_resp = connector
        .client()?
        .put_item()
        .table_name(&table_name)
        .set_item(Some(item))
        .send()
        .await;

    // Check if PutItem was successful
    assert!(put_resp.is_ok(), "PutItem failed: {:?}", put_resp.err());

    // Optional: Add a GetItem check here if desired

    Ok(())
}

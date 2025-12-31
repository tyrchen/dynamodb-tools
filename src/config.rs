use crate::error::{DynamoToolsError, Result};
use aws_sdk_dynamodb::{
    operation::create_table::CreateTableInput,
    types::{
        AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType,
        LocalSecondaryIndex, Projection, ProjectionType, ProvisionedThroughput,
        ScalarAttributeType,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs::File, io::BufReader, path::Path};

/// Represents the main configuration loaded from a YAML file.
///
/// This struct defines the overall settings for connecting to DynamoDB,
/// including endpoint, region, and definitions for one or more tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    /// AWS region to target. Defaults to "us-east-1" if not specified.
    #[serde(default = "default_aws_region")]
    pub region: String,
    /// Optional local endpoint URL (e.g., "http://localhost:8000" for DynamoDB Local).
    /// If provided, the connector targets this endpoint and uses test credentials.
    #[serde(default)]
    pub endpoint: Option<String>,
    /// If `true` and `endpoint` is set, created tables will be deleted
    /// when the `DynamodbConnector` is dropped (requires `test_utils` feature).
    #[serde(default)]
    pub delete_on_exit: bool,
    /// A list of table schemas to be managed by the connector.
    #[serde(default)]
    pub tables: Vec<TableInfo>,
}

/// Defines the detailed schema for a single DynamoDB table.
///
/// Used within the `tables` list in [`TableConfig`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// The base name of the table. A unique ID will be appended
    /// by the connector upon creation (e.g., `my_table` becomes `my_table-unique_id`).
    /// This base name is used to retrieve the actual table name later.
    pub table_name: String,
    /// The primary partition key attribute definition.
    pub pk: TableAttr,
    /// Optional primary sort key attribute definition.
    #[serde(default)]
    pub sk: Option<TableAttr>,
    /// Additional attribute definitions beyond the primary keys.
    /// PK and SK attributes are automatically included, no need to repeat here.
    #[serde(default)]
    pub attrs: Vec<TableAttr>,
    /// Global Secondary Index definitions.
    #[serde(default)]
    pub gsis: Vec<TableGsi>,
    /// Local Secondary Index definitions.
    #[serde(default)]
    pub lsis: Vec<TableLsi>,
    /// Optional provisioned throughput settings. If `None`, uses Pay-Per-Request billing.
    #[serde(default)]
    pub throughput: Option<Throughput>,
    /// Optional path to a JSON file containing an array of items to seed into the table after creation.
    #[serde(default)]
    pub seed_data_file: Option<String>,
}

/// Defines provisioned throughput settings (read/write capacity units).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Throughput {
    /// Read Capacity Units (RCU).
    pub read: i64,
    /// Write Capacity Units (WCU).
    pub write: i64,
}

/// Defines a single DynamoDB attribute (name and type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableAttr {
    /// The name of the attribute.
    pub name: String,
    /// The DynamoDB type of the attribute (S, N, B).
    #[serde(rename = "type")]
    pub attr_type: AttrType,
}

/// Represents the possible DynamoDB scalar attribute types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttrType {
    /// String type.
    S,
    /// Number type.
    N,
    /// Binary type.
    B,
}

/// Defines a Global Secondary Index (GSI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableGsi {
    /// The name of the GSI.
    pub name: String,
    /// The partition key attribute for the GSI.
    pub pk: TableAttr,
    /// Optional sort key attribute for the GSI.
    #[serde(default)]
    pub sk: Option<TableAttr>,
    /// Attributes to project into the GSI (only used if projection type is INCLUDE).
    #[serde(default)]
    pub attrs: Vec<String>,
    /// Optional provisioned throughput for the GSI.
    #[serde(default)]
    pub throughput: Option<Throughput>,
}

/// Defines a Local Secondary Index (LSI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableLsi {
    /// The name of the LSI.
    pub name: String,
    /// The partition key attribute (must be the same as the table's PK).
    pub pk: TableAttr,
    /// The sort key attribute for the LSI.
    pub sk: TableAttr,
    /// Attributes to project into the LSI (only used if projection type is INCLUDE).
    #[serde(default)]
    pub attrs: Vec<String>,
}

// Internal helper function for default region
fn default_aws_region() -> String {
    "us-east-1".to_string()
}

impl From<AttrType> for ScalarAttributeType {
    fn from(attr_type: AttrType) -> Self {
        match attr_type {
            AttrType::S => ScalarAttributeType::S,
            AttrType::N => ScalarAttributeType::N,
            AttrType::B => ScalarAttributeType::B,
        }
    }
}

impl From<TableAttr> for AttributeDefinition {
    fn from(attr: TableAttr) -> Self {
        let attr_type = attr.attr_type.into();
        AttributeDefinition::builder()
            .attribute_name(attr.name)
            .attribute_type(attr_type)
            .build()
            .unwrap()
    }
}

impl TableAttr {
    fn to_pk(&self) -> KeySchemaElement {
        KeySchemaElement::builder()
            .attribute_name(self.name.clone())
            .key_type(KeyType::Hash)
            .build()
            .unwrap()
    }

    fn to_sk(&self) -> KeySchemaElement {
        KeySchemaElement::builder()
            .attribute_name(self.name.clone())
            .key_type(KeyType::Range)
            .build()
            .unwrap()
    }
}

impl From<TableGsi> for GlobalSecondaryIndex {
    fn from(gsi: TableGsi) -> Self {
        let pk = gsi.pk.to_pk();
        let sk = gsi.sk.map(|sk| sk.to_sk());

        let key_schema = if let Some(sk) = sk {
            vec![pk, sk]
        } else {
            vec![pk]
        };

        let mut builder = GlobalSecondaryIndex::builder()
            .set_key_schema(Some(key_schema))
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::Include)
                    .set_non_key_attributes(Some(gsi.attrs))
                    .build(),
            )
            .index_name(gsi.name);

        if let Some(throughput) = gsi.throughput {
            let pt = ProvisionedThroughput::builder()
                .read_capacity_units(throughput.read)
                .write_capacity_units(throughput.write)
                .build()
                .unwrap();
            builder = builder.provisioned_throughput(pt);
        }
        builder.build().unwrap()
    }
}

impl From<TableLsi> for LocalSecondaryIndex {
    fn from(lsi: TableLsi) -> Self {
        let pk = lsi.pk.to_pk();
        let sk = lsi.sk.to_sk();
        let key_schema = vec![pk, sk];
        let projection = if lsi.attrs.is_empty() {
            Projection::builder()
                .projection_type(ProjectionType::All)
                .build()
        } else {
            Projection::builder()
                .projection_type(ProjectionType::Include)
                .set_non_key_attributes(Some(lsi.attrs))
                .build()
        };
        LocalSecondaryIndex::builder()
            .set_key_schema(Some(key_schema))
            .projection(projection)
            .index_name(lsi.name)
            .build()
            .unwrap()
    }
}

impl TryFrom<TableInfo> for CreateTableInput {
    type Error = DynamoToolsError;

    fn try_from(config: TableInfo) -> Result<Self> {
        // Use a HashMap to collect unique attribute definitions by name
        let mut attribute_map: HashMap<String, TableAttr> = HashMap::new();

        // 1. Add base table keys
        attribute_map.insert(config.pk.name.clone(), config.pk.clone());
        if let Some(ref sk) = config.sk {
            attribute_map.insert(sk.name.clone(), sk.clone());
        }

        // 2. Add GSI keys
        for gsi in &config.gsis {
            attribute_map.insert(gsi.pk.name.clone(), gsi.pk.clone());
            if let Some(ref sk) = gsi.sk {
                attribute_map.insert(sk.name.clone(), sk.clone());
            }
        }

        // 4. Add LSI keys
        for lsi in &config.lsis {
            attribute_map.insert(lsi.sk.name.clone(), lsi.sk.clone());
        }

        // Convert the unique attributes to AttributeDefinition vector
        let final_attrs: Vec<AttributeDefinition> = attribute_map
            .into_values()
            .map(AttributeDefinition::from)
            .collect();

        // --- Key Schema (remains the same) ---
        let pk_schema = config.pk.to_pk();
        let sk_schema = config.sk.as_ref().map(|sk| sk.to_sk());
        let key_schema = if let Some(sk) = sk_schema {
            vec![pk_schema, sk]
        } else {
            vec![pk_schema]
        };
        // --- End Key Schema ---

        // --- GSI/LSI Conversion (remains the same) ---
        let gsis: Vec<GlobalSecondaryIndex> = config
            .gsis
            .into_iter()
            .map(GlobalSecondaryIndex::from)
            .collect();
        let lsis: Vec<LocalSecondaryIndex> = config
            .lsis
            .into_iter()
            .map(LocalSecondaryIndex::from)
            .collect();
        // --- End GSI/LSI ---

        // --- Build CreateTableInput ---
        let mut builder = CreateTableInput::builder()
            .table_name(config.table_name)
            .set_key_schema(Some(key_schema))
            .set_attribute_definitions(Some(final_attrs)); // Use the final collected attrs

        if !gsis.is_empty() {
            builder = builder.set_global_secondary_indexes(Some(gsis));
        }
        if !lsis.is_empty() {
            builder = builder.set_local_secondary_indexes(Some(lsis));
        }

        match config.throughput {
            Some(throughput) => {
                let pt = ProvisionedThroughput::builder()
                    .read_capacity_units(throughput.read)
                    .write_capacity_units(throughput.write)
                    .build()
                    .map_err(|e| {
                        DynamoToolsError::Internal(format!(
                            "Failed to build ProvisionedThroughput: {}",
                            e
                        ))
                    })?;
                builder = builder.provisioned_throughput(pt);
            }
            None => {
                builder = builder.billing_mode(BillingMode::PayPerRequest);
            }
        }
        // --- End Build ---

        builder.build().map_err(DynamoToolsError::AwsSdkConfig)
    }
}

impl TableConfig {
    /// Loads [`TableConfig`] from a YAML file.
    ///
    /// Expects a top-level structure with keys like `region`, `endpoint`, `tables` (a list).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file cannot be read ([`DynamoToolsError::ConfigRead`])
    /// or if the YAML content cannot be parsed ([`DynamoToolsError::ConfigParse`]).
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_string();
        let file =
            File::open(path_ref).map_err(|e| DynamoToolsError::ConfigRead(path_str.clone(), e))?;
        let reader = BufReader::new(file);
        let config = serde_yml::from_reader(reader)
            .map_err(|e| DynamoToolsError::ConfigParse(path_str, e))?;
        Ok(config)
    }

    /// Creates a new `TableConfig` programmatically.
    pub fn new(
        region: String,
        endpoint: Option<String>,
        delete_on_exit: bool,
        tables: Vec<TableInfo>,
    ) -> Self {
        let delete_on_exit = if endpoint.is_some() {
            delete_on_exit
        } else {
            false
        };

        Self {
            region,
            endpoint,
            delete_on_exit,
            tables,
        }
    }
}

impl TableInfo {
    /// Loads [`TableInfo`] directly from a YAML file.
    ///
    /// Generally, it's preferred to load the full [`TableConfig`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file cannot be read ([`DynamoToolsError::ConfigRead`])
    /// or if the YAML content cannot be parsed ([`DynamoToolsError::ConfigParse`]).
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_string();
        let file =
            File::open(path_ref).map_err(|e| DynamoToolsError::ConfigRead(path_str.clone(), e))?;
        let reader = BufReader::new(file);
        let info = serde_yml::from_reader(reader)
            .map_err(|e| DynamoToolsError::ConfigParse(path_str, e))?;
        Ok(info)
    }

    /// Loads [`TableInfo`] directly from a YAML string.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the YAML string cannot be parsed ([`DynamoToolsError::ConfigParse`]).
    ///
    /// # Example
    ///
    /// ```rust
    /// use dynamodb_tools::{TableInfo, AttrType};
    ///
    /// let yaml_data = r#"
    /// table_name: my_simple_table
    /// pk:
    ///   name: item_id
    ///   type: S
    /// "#;
    ///
    /// let table_info = TableInfo::load(yaml_data).unwrap();
    ///
    /// assert_eq!(table_info.table_name, "my_simple_table");
    /// assert_eq!(table_info.pk.name, "item_id");
    /// assert_eq!(table_info.pk.attr_type, AttrType::S);
    /// assert!(table_info.sk.is_none());
    /// ```
    pub fn load(s: &str) -> Result<Self> {
        let info = serde_yml::from_str(s)
            .map_err(|e| DynamoToolsError::ConfigParse("string input".to_string(), e))?;
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_could_be_loaded() {
        let config = TableConfig::load_from_file("fixtures/dev.yml").unwrap();
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.endpoint, Some("http://localhost:8000".to_string()));
        assert!(config.delete_on_exit);
        assert!(!config.tables.is_empty());

        let info = config.tables[0].clone();
        assert_eq!(info.table_name, "users");
        assert_eq!(info.pk.name, "pk");
        assert_eq!(info.pk.attr_type, AttrType::S);
        assert!(info.sk.is_some());
        assert_eq!(info.gsis.len(), 1);
        assert_eq!(info.lsis.len(), 1);
    }

    #[test]
    fn table_info_could_be_loaded() {
        let info = TableInfo::load_from_file("fixtures/info.yml").unwrap();
        assert_eq!(info.table_name, "users");
        assert_eq!(info.pk.name, "pk");
        assert_eq!(info.pk.attr_type, AttrType::S);
    }
}

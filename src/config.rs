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
use std::{fs::File, io::BufReader, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub table_name: String,
    /// AWS region, if provided, dynamodb connector will connect to the region
    #[serde(default = "default_aws_region")]
    pub region: String,
    /// local endpoints, if provided, dynamodb connector will connect dynamodb local
    #[serde(default)]
    pub endpoint: Option<String>,
    /// drop table when connector is dropped. Would only work if local_endpoint is provided
    #[serde(default)]
    pub delete_on_exit: bool,
    /// table info
    #[serde(default)]
    pub info: Option<TableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub pk: TableAttr,
    #[serde(default)]
    pub sk: Option<TableAttr>,
    #[serde(default)]
    pub attrs: Vec<TableAttr>,
    #[serde(default)]
    pub gsis: Vec<TableGsi>,
    #[serde(default)]
    pub lsis: Vec<TableLsi>,
    #[serde(default)]
    pub throughput: Option<Throughput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Throughput {
    pub read: i64,
    pub write: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableAttr {
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: AttrType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttrType {
    S,
    N,
    B,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableGsi {
    pub name: String,
    pub pk: TableAttr,
    #[serde(default)]
    pub sk: Option<TableAttr>,
    #[serde(default)]
    pub attrs: Vec<String>,
    #[serde(default)]
    pub throughput: Option<Throughput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableLsi {
    pub name: String,
    // must be the same as the pk of the table
    pub pk: TableAttr,
    pub sk: TableAttr,
    #[serde(default)]
    pub attrs: Vec<String>,
}

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
        let pk = config.pk.to_pk();
        let sk = config.sk.as_ref().map(|sk| sk.to_sk());

        let key_schema = if let Some(sk) = sk {
            vec![pk, sk]
        } else {
            vec![pk]
        };

        // add pk and sk to attrs
        let mut attrs = config.attrs.clone();
        attrs.push(config.pk);
        if let Some(sk) = config.sk {
            attrs.push(sk);
        }
        let attrs: Vec<_> = attrs.into_iter().map(AttributeDefinition::from).collect();

        // Add explicit types for collected vectors
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

        let mut builder = CreateTableInput::builder()
            .table_name(config.table_name)
            .set_key_schema(Some(key_schema))
            .set_attribute_definitions(Some(attrs));

        // Only set indexes if the vectors are not empty
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

        builder.build().map_err(DynamoToolsError::AwsSdkConfig)
    }
}

impl TableConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_string();
        let file =
            File::open(path_ref).map_err(|e| DynamoToolsError::ConfigRead(path_str.clone(), e))?;
        let reader = BufReader::new(file);
        let config = serde_yaml::from_reader(reader)
            .map_err(|e| DynamoToolsError::ConfigParse(path_str, e))?;
        Ok(config)
    }

    pub fn new(
        table_name: String,
        region: String,
        endpoint: Option<String>,
        delete_on_exit: bool,
        info: Option<TableInfo>,
    ) -> Self {
        let delete_on_exit = if endpoint.is_some() {
            delete_on_exit
        } else {
            false
        };

        Self {
            table_name,
            region,
            endpoint,
            delete_on_exit,
            info,
        }
    }
}

impl TableInfo {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_string();
        let file =
            File::open(path_ref).map_err(|e| DynamoToolsError::ConfigRead(path_str.clone(), e))?;
        let reader = BufReader::new(file);
        let info = serde_yaml::from_reader(reader)
            .map_err(|e| DynamoToolsError::ConfigParse(path_str, e))?;
        Ok(info)
    }

    pub fn load(s: &str) -> Result<Self> {
        let info = serde_yaml::from_str(s)
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
        assert_eq!(config.table_name, "users");
        assert_eq!(config.endpoint, Some("http://localhost:8000".to_string()));
        assert!(config.delete_on_exit);
        assert!(config.info.is_some());

        let info = config.info.unwrap();
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

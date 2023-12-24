use anyhow::Result;
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
    /// local endpoints, if provided, dynamodb connector will connect dynamodb local
    #[serde(default)]
    pub(crate) local_endpoint: Option<String>,
    /// drop table when connector is dropped. Would only work if local_endpoint is provided
    #[serde(default)]
    pub(crate) delete_on_exit: bool,
    /// table info
    pub(crate) info: Option<TableInfo>,
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
            .expect("attr should be valid")
    }
}

impl TableAttr {
    fn to_pk(&self) -> KeySchemaElement {
        KeySchemaElement::builder()
            .attribute_name(self.name.clone())
            .key_type(KeyType::Hash)
            .build()
            .expect("pk should be valid")
    }

    fn to_sk(&self) -> KeySchemaElement {
        KeySchemaElement::builder()
            .attribute_name(self.name.clone())
            .key_type(KeyType::Range)
            .build()
            .expect("sk should be valid")
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
            // .provisioned_throughput(pt)
            .index_name(gsi.name);

        if let Some(throughput) = gsi.throughput {
            let pt = ProvisionedThroughput::builder()
                .read_capacity_units(throughput.read)
                .write_capacity_units(throughput.write)
                .build()
                .expect("throughput should be valid");
            builder = builder.provisioned_throughput(pt);
        }
        builder.build().expect("gsi should be valid")
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
            .expect("lsi should be valid")
    }
}

impl From<TableInfo> for CreateTableInput {
    fn from(config: TableInfo) -> Self {
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
        let attrs = attrs.into_iter().map(AttributeDefinition::from).collect();

        let gsis = config
            .gsis
            .into_iter()
            .map(GlobalSecondaryIndex::from)
            .collect();

        let lsis = config
            .lsis
            .into_iter()
            .map(LocalSecondaryIndex::from)
            .collect();

        let mut builder = CreateTableInput::builder()
            .table_name(config.table_name)
            .set_key_schema(Some(key_schema))
            .set_attribute_definitions(Some(attrs))
            .set_global_secondary_indexes(Some(gsis))
            .set_local_secondary_indexes(Some(lsis));

        match config.throughput {
            Some(throughput) => {
                let pt = ProvisionedThroughput::builder()
                    .read_capacity_units(throughput.read)
                    .write_capacity_units(throughput.write)
                    .build()
                    .expect("throughput should be valid");
                builder = builder.provisioned_throughput(pt);
            }
            None => {
                builder = builder.billing_mode(BillingMode::PayPerRequest);
            }
        }

        builder.build().expect("table info should be valid")
    }
}

impl TableConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    pub fn new(
        table_name: String,
        local_endpoint: Option<String>,
        delete_on_exit: bool,
        info: Option<TableInfo>,
    ) -> Self {
        let delete_on_exit = if local_endpoint.is_some() {
            delete_on_exit
        } else {
            false
        };

        Self {
            table_name,
            local_endpoint,
            delete_on_exit,
            info,
        }
    }
}

impl TableInfo {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    pub fn load(s: &str) -> Result<Self> {
        let config = serde_yaml::from_str(s)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_could_be_loaded() {
        let config = TableConfig::load_from_file("fixtures/dev.yml").unwrap();

        let info = config.info.expect("info should be present");

        assert_eq!(config.table_name, "users");
        assert_eq!(info.pk.name, "pk");
        assert_eq!(info.pk.attr_type, AttrType::S);

        let input = CreateTableInput::try_from(info).unwrap();
        assert_eq!(input.attribute_definitions().len(), 5);
        assert_eq!(input.global_secondary_indexes().len(), 1);
        assert_eq!(input.local_secondary_indexes().len(), 1);
    }

    #[test]
    fn table_info_could_be_loaded() {
        let info = TableInfo::load_from_file("fixtures/info.yml").unwrap();

        assert_eq!(info.pk.name, "pk");
        assert_eq!(info.pk.attr_type, AttrType::S);
    }
}

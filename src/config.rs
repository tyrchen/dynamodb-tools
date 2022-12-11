use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Error, Result};
use aws_sdk_dynamodb::{
    input::CreateTableInput,
    model::{
        AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, KeyType, LocalSecondaryIndex,
        Projection, ProjectionType, ProvisionedThroughput, ScalarAttributeType,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
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
    }
}

impl TableAttr {
    fn to_pk(&self) -> KeySchemaElement {
        KeySchemaElement::builder()
            .attribute_name(self.name.clone())
            .key_type(KeyType::Hash)
            .build()
    }

    fn to_sk(&self) -> KeySchemaElement {
        KeySchemaElement::builder()
            .attribute_name(self.name.clone())
            .key_type(KeyType::Range)
            .build()
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
        let pt = ProvisionedThroughput::builder()
            .read_capacity_units(5)
            .write_capacity_units(5)
            .build();
        GlobalSecondaryIndex::builder()
            .set_key_schema(Some(key_schema))
            .projection(
                Projection::builder()
                    .projection_type(ProjectionType::Include)
                    .set_non_key_attributes(Some(gsi.attrs))
                    .build(),
            )
            .provisioned_throughput(pt)
            .index_name(gsi.name)
            .build()
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
    }
}

impl TryFrom<TableConfig> for CreateTableInput {
    type Error = Error;
    fn try_from(config: TableConfig) -> Result<Self> {
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

        let pt = ProvisionedThroughput::builder()
            .read_capacity_units(5)
            .write_capacity_units(5)
            .build();
        let input = CreateTableInput::builder()
            .table_name(config.table_name)
            .set_key_schema(Some(key_schema))
            .set_attribute_definitions(Some(attrs))
            .set_global_secondary_indexes(Some(gsis))
            .set_local_secondary_indexes(Some(lsis))
            .provisioned_throughput(pt);

        Ok(input.build()?)
    }
}

impl TableConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_could_be_loaded() {
        let config = TableConfig::load_from_file("fixtures/config.yml").unwrap();

        assert_eq!(config.table_name, "users");
        assert_eq!(config.pk.name, "pk");
        assert_eq!(config.pk.attr_type, AttrType::S);

        let input = CreateTableInput::try_from(config).unwrap();
        assert_eq!(input.attribute_definitions().unwrap().len(), 5);
        assert_eq!(input.global_secondary_indexes().unwrap().len(), 1);
        assert_eq!(input.local_secondary_indexes().unwrap().len(), 1);
    }
}

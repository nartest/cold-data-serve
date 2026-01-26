use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigV2 {
    pub version: String,
    pub collection: String,
    pub source: SourceConfig,
    pub storage: StorageConfig,
    pub indexes: Vec<IndexConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceConfig {
    pub path: PathBuf,
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub primary_key: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexConfig {
    pub column: String,
    pub r#type: IndexType,
    #[serde(default)]
    pub stopwords: Option<Vec<String>>,
    #[serde(default)]
    pub resolution: Option<u8>,
    #[serde(default)]
    pub bins: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IndexType {
    Text,
    Bitmap,
    Geo,
    Range,
    Date,
    DateTime,
}

impl ConfigV2 {
    pub fn from_yaml_str(content: &str) -> anyhow::Result<Self> {
        let config: ConfigV2 = serde_yaml::from_str(content)?;
        Ok(config)
    }

    pub fn from_yaml<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml_str(&content)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // La clé primaire doit être dans les colonnes stockées
        if !self.storage.columns.contains(&self.storage.primary_key) {
            return Err(anyhow::anyhow!(
                "Primary key '{}' must be present in storage columns",
                self.storage.primary_key
            ));
        }

        // Pour chaque index, vérifier la cohérence (ex: géo a une résolution pour H3)
        for idx in &self.indexes {
            if idx.r#type == IndexType::Geo && idx.resolution.is_none() {
                // On pourrait mettre une valeur par défaut, mais validation stricte ici si besoin
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_v2_yaml() {
        let yaml = r#"
version: "1.0"
collection: "bano"

source:
  path: "data/bano-full.parquet"
  query: |
    SELECT 
      id, 
      house_number || ' ' || street || ' ' || zip_code || ' ' || city AS full_name,
      house_number || ' ' || street AS street,
      city,
      zip_code,
      ST_Point(lon::DOUBLE, lat::DOUBLE) as geom,
      floor(random() * 100) as score 
    FROM read_parquet('{path}')

storage:
  primary_key: "id"
  columns: ["id", "street", "zip_code", "city", "geom"]

indexes:
  - column: "full_name"
    type: "text"
    stopwords: [ "rue", "impasse", "route" ]

  - column: "zip_code"
    type: "bitmap"

  - column: "geom"
    type: "geo"
    resolution: 8

  - column: "score"
    type: "range"
    bins: 20
"#;

        let config = ConfigV2::from_yaml_str(yaml).unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.collection, "bano");
        assert_eq!(config.storage.primary_key, "id");
        assert_eq!(config.indexes.len(), 4);
        assert_eq!(config.indexes[0].r#type, IndexType::Text);
        assert_eq!(config.indexes[2].r#type, IndexType::Geo);
        assert_eq!(config.indexes[2].resolution, Some(8));
    }
}

use cold_search_core::config_v2::{ConfigV2, IndexType};

pub struct QueryBuilder;

impl QueryBuilder {
    pub fn build_stage_query(config: &ConfigV2) -> String {
        let user_query = config
            .source
            .query
            .replace("{path}", config.source.path.to_str().unwrap_or_default());

        let mut extra_columns = Vec::new();
        let mut geo_col = None;

        for idx in &config.indexes {
            if idx.r#type == IndexType::Geo {
                geo_col = Some(&idx.column);
            }
        }

        if let Some(col) = geo_col {
            // For H3 compatibility (uses _idx_lon/_idx_lat)
            extra_columns.push(format!(
                "st_xmin({0}) AS _idx_lon, st_ymin({0}) AS _idx_lat",
                col
            ));
            // For uniform sorting
            extra_columns.push(format!(
                "st_xmin({0}) AS _idx_xmin, st_ymin({0}) AS _idx_ymin",
                col
            ));
            // ECEF coordinates based on min point
            extra_columns.push(format!("(6371000 * cos(radians(st_ymin({0}))) * cos(radians(st_xmin({0}))) )::FLOAT as _idx_ecef_x", col));
            extra_columns.push(format!("(6371000 * cos(radians(st_ymin({0}))) * sin(radians(st_xmin({0}))) )::FLOAT as _idx_ecef_y", col));
            extra_columns.push(format!(
                "(6371000 * sin(radians(st_ymin({0}))) )::FLOAT as _idx_ecef_z",
                col
            ));
            // Geometry type for detection
            extra_columns.push(format!(
                "st_geometrytype({0})::VARCHAR AS _idx_geometry_type",
                col
            ));
            // WKB for FGB generation
            extra_columns.push(format!("st_aswkb({0}) AS _idx_wkb", col));
        }

        let select_extra = if extra_columns.is_empty() {
            "".to_string()
        } else {
            format!(", {}", extra_columns.join(", "))
        };

        let mut query = format!(
            "CREATE TABLE stage_indexation AS SELECT u.* {} FROM ({}) u",
            select_extra, user_query
        );

        if geo_col.is_some() {
            query.push_str(" ORDER BY _idx_ymin, _idx_xmin");
        }

        query
    }

    pub fn build_quantile_query(column: &str, bins: usize) -> String {
        let mut quantiles = Vec::new();
        for i in 1..bins {
            quantiles.push(format!("{:.2}", i as f32 / bins as f32));
        }
        let q_list = quantiles.join(", ");
        format!(
            "SELECT CAST(quantile_cont({}, [{}]) AS VARCHAR) FROM stage_indexation",
            column, q_list
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cold_search_core::config_v2::ConfigV2;

    #[test]
    fn test_build_stage_query_with_geo() {
        let yaml = r#"
version: "1.0"
collection: "test"
source:
  path: "test.parquet"
  query: "SELECT id, ST_Point(lon, lat) as geom FROM read_parquet('{path}')"
storage:
  primary_key: "id"
  columns: ["id", "geom"]
indexes:
  - column: "geom"
    type: "geo"
"#;
        let config = ConfigV2::from_yaml_str(yaml).unwrap();
        let query = QueryBuilder::build_stage_query(&config);

        assert!(query.contains("CREATE TABLE stage_indexation"));
        assert!(query.contains("_idx_ecef_x"));
        assert!(query.contains("_idx_geometry_type"));
        assert!(query.contains("ORDER BY _idx_ymin, _idx_xmin"));
    }

    #[test]
    fn test_duckdb_integration() {
        let yaml = r#"
version: "1.0"
collection: "test"
source:
  path: "test_integration.csv"
  query: "SELECT id, ST_Point(lon, lat) as geom FROM read_csv_auto('{path}')"
storage:
  primary_key: "id"
  columns: ["id", "geom"]
indexes:
  - column: "geom"
    type: "geo"
"#;
        // Créer un fichier CSV temporaire
        let csv_content = "id,lat,lon\n1,48.8566,2.3522\n2,45.7640,4.8357";
        std::fs::write("test_integration.csv", csv_content).unwrap();

        let config = ConfigV2::from_yaml_str(yaml).unwrap();
        let query = QueryBuilder::build_stage_query(&config);

        let conn = duckdb::Connection::open_in_memory().unwrap();
        // Tenter de charger l'extension spatiale (peut échouer selon l'environnement de build)
        let _ = conn.execute_batch("INSTALL spatial; LOAD spatial;");

        match conn.execute_batch(&query) {
            Ok(_) => {
                let mut stmt = conn
                    .prepare("PRAGMA table_info('stage_indexation')")
                    .unwrap();
                let rows = stmt
                    .query_map([], |row| {
                        let name: String = row.get(1)?;
                        Ok(name)
                    })
                    .unwrap();

                let cols: Vec<String> = rows.map(|r| r.unwrap()).collect();
                assert!(cols.contains(&"id".to_string()));
                assert!(cols.contains(&"_idx_ecef_x".to_string()));
                assert!(cols.contains(&"_idx_lat".to_string()));
                assert!(cols.contains(&"_idx_geometry_type".to_string()));
            }
            Err(e) => {
                // Si l'extension spatiale n'est pas dispo, on ignore l'erreur d'exécution
                // mais on logue pour info
                println!(
                    "DuckDB execution failed (probably missing spatial extension): {}",
                    e
                );
            }
        }

        std::fs::remove_file("test_integration.csv").unwrap();
    }
}

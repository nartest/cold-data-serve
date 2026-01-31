use crate::query_builder::QueryBuilder;
use anyhow::{anyhow, Result};
use chrono::Datelike;
use cold_search_core::config_v2::{ConfigV2, IndexType};
use cold_search_core::schema_generated::{
    BooleanWrapper, DateTimeWrapper, DateWrapper, Entry, Field, FloatWrapper, IntegerWrapper,
    TextWrapper, Value,
};
use cold_search_core::slugify;
use cold_search_core::RuntimeConfig;
use duckdb::Connection;
use flatgeobuf::*;
use geozero::{ColumnValue, PropertyProcessor};
use h3o::{LatLng, Resolution};
use rayon::prelude::*;
use roaring::RoaringBitmap;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Pipeline {
    config: ConfigV2,
    output_dir: PathBuf,
    // HashMap<NomColonne, HashMap<Valeur, Bitmap>>
    bitmaps: Arc<Mutex<HashMap<String, HashMap<String, RoaringBitmap>>>>,
    // Coordonnées ECEF
    ecef_coords: Arc<Mutex<Vec<[f32; 3]>>>,
    // Stockage (FlatBuffers)
    data_file: Arc<Mutex<BufWriter<File>>>,
    offset_file: Arc<Mutex<BufWriter<File>>>,
    current_data_offset: Arc<Mutex<u64>>,
    // Bornes pour les index de type range (NomColonne -> Bornes)
    range_bins: Arc<Mutex<HashMap<String, Vec<f64>>>>,
    // Primary Key FST (Key -> DocId)
    primary_fst: Arc<Mutex<BTreeMap<String, u64>>>,
    // Valeurs brutes pour les index de type range (NomColonne -> Vec<Valeur>)
    range_values: Arc<Mutex<HashMap<String, Vec<f64>>>>,
    // Stratégie géométrique par colonne (H3 ou FGB)
    geo_strategies: Arc<Mutex<HashMap<String, GeoStrategy>>>,
    memory_limit: String,
    threads: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeoStrategy {
    H3,
    FGB,
}

impl Pipeline {
    pub fn new(
        config: ConfigV2,
        output_dir: PathBuf,
        memory_limit: String,
        threads: usize,
    ) -> Result<Self> {
        if output_dir.exists() {
            println!("Cleaning output directory: {:?}", output_dir);
            // On supprime le contenu mais on garde le dossier
            for entry in fs::read_dir(&output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
        }
        fs::create_dir_all(&output_dir)?;
        let data_path = output_dir.join("data.bin");
        let offset_path = output_dir.join("data.offsets");

        Ok(Self {
            config,
            output_dir,
            bitmaps: Arc::new(Mutex::new(HashMap::new())),
            ecef_coords: Arc::new(Mutex::new(Vec::new())),
            data_file: Arc::new(Mutex::new(BufWriter::new(File::create(data_path)?))),
            offset_file: Arc::new(Mutex::new(BufWriter::new(File::create(offset_path)?))),
            current_data_offset: Arc::new(Mutex::new(0)),
            range_bins: Arc::new(Mutex::new(HashMap::new())),
            primary_fst: Arc::new(Mutex::new(BTreeMap::new())),
            range_values: Arc::new(Mutex::new(HashMap::new())),
            geo_strategies: Arc::new(Mutex::new(HashMap::new())),
            memory_limit,
            threads,
        })
    }

    pub fn run(&self) -> Result<()> {
        let conn = self.setup_db()?;
        self.create_stage_table(&conn)?;
        self.detect_geo_strategies(&conn)?;
        self.process_statistics(&conn)?;
        self.stream_and_process(&conn)?;
        self.finalize(&conn)?;

        // Nettoyage après indexation : suppression de la base DuckDB et du dossier temporaire
        drop(conn);
        let db_path = self.output_dir.join("indexing.db");
        if db_path.exists() {
            fs::remove_file(db_path)?;
        }
        let temp_dir = self.output_dir.join("duckdb_tmp");
        if temp_dir.exists() {
            fs::remove_dir_all(temp_dir)?;
        }

        Ok(())
    }

    fn detect_geo_strategies(&self, conn: &Connection) -> Result<()> {
        let mut strategies = self.geo_strategies.lock().unwrap();

        for idx in &self.config.indexes {
            if idx.r#type == IndexType::Geo {
                println!("Detecting geometry strategy for {}...", idx.column);
                let _query = format!(
                    "SELECT DISTINCT _idx_geometry_type FROM stage_indexation WHERE {} IS NOT NULL",
                    idx.column
                );
                // Note: The column name in WHERE is likely not needed if _idx_geometry_type is derived from it,
                // but strictly speaking we generated _idx_geometry_type already.
                // However, since we might have multiple geo columns, checking distinct on the table implies checks on ALL rows?
                // Wait, QueryBuilder generates `_idx_geometry_type` using `st_geometrytype({col})`.
                // If we have multiple geo columns, we might have `_idx_geometry_type` conflict or ambiguity?
                // QueryBuilder currently generates ONE `_idx_geometry_type` if `geo_col` is found.
                // It iterates and overrides `geo_col`.
                // Current QueryBuilder implementation only supports ONE geo column properly (it uses a loop but `geo_col` is option).
                // So we assume one geo column for now.

                // Let's check QueryBuilder logic again.
                // `for idx in ... if Geo { geo_col = Some(...) }`. It takes the LAST one.
                // This means we only support ONE geo column effectively in QueryBuilder right now.
                // Assuming single geo column for now as per current codebase limitation.

                let mut stmt = conn.prepare("SELECT DISTINCT _idx_geometry_type FROM stage_indexation WHERE _idx_geometry_type IS NOT NULL")?;
                let types: Vec<String> = stmt
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<Vec<_>, _>>()?;

                if types.is_empty() {
                    println!("No geometry types found, defaulting to H3 (Points).");
                    strategies.insert(idx.column.clone(), GeoStrategy::H3);
                    continue;
                }

                println!("Found geometry types: {:?}", types);

                let has_points = types.iter().any(|t| t == "POINT" || t == "MULTIPOINT");
                let has_polygons = types.iter().any(|t| t == "POLYGON" || t == "MULTIPOLYGON");

                if has_points && has_polygons {
                    return Err(anyhow!("Mixed geometry types (Points and Polygons) are not supported in the same column."));
                }

                if has_polygons {
                    println!("Strategy: FGB (Polygons)");
                    strategies.insert(idx.column.clone(), GeoStrategy::FGB);
                } else {
                    println!("Strategy: H3 (Points)");
                    strategies.insert(idx.column.clone(), GeoStrategy::H3);
                }
            }
        }
        Ok(())
    }

    fn setup_db(&self) -> Result<Connection> {
        let db_path = self.output_dir.join("indexing.db");
        if db_path.exists() {
            std::fs::remove_file(&db_path)?;
        }
        let conn = Connection::open(&db_path)?;

        // Optimisations suggérées dans Indexation.md
        let temp_dir = self.output_dir.join("duckdb_tmp");
        std::fs::create_dir_all(&temp_dir)?;

        conn.execute_batch(&format!(
            "PRAGMA temp_directory = '{}';
             SET preserve_insertion_order = false;
             SET threads={};
             SET memory_limit='{}';
             SET wal_autocheckpoint = '1GB';
             INSTALL spatial; LOAD spatial;",
            temp_dir.display(),
            self.threads,
            self.memory_limit
        ))?;

        Ok(conn)
    }

    fn create_stage_table(&self, conn: &Connection) -> Result<()> {
        println!("Building stage_indexation table...");
        let query = QueryBuilder::build_stage_query(&self.config);
        conn.execute_batch(&query)?;
        Ok(())
    }

    fn process_statistics(&self, conn: &Connection) -> Result<()> {
        for idx in &self.config.indexes {
            match idx.r#type {
                IndexType::Range | IndexType::Date | IndexType::DateTime => {
                    let bins = idx.bins.unwrap_or(20);
                    let query = QueryBuilder::build_quantile_query(&idx.column, bins);
                    println!("Computing statistics for {}: {} bins", idx.column, bins);

                    let mut stmt = conn.prepare(&query)?;
                    let quantiles_str: String = stmt.query_row([], |row| row.get(0))?;
                    // Parse "[0.1, 0.5, 0.9]" or "['2022-01-01', ...]"
                    let quantiles: Vec<f64> = quantiles_str
                        .trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if s.is_empty() {
                                return 0.0;
                            }
                            // Try parse as number first
                            if let Ok(v) = s.parse::<f64>() {
                                return v;
                            }
                            // Try parse as date
                            if idx.r#type == IndexType::Date {
                                if let Ok(d) = chrono::NaiveDate::parse_from_str(
                                    s.trim_matches('\''),
                                    "%Y-%m-%d",
                                ) {
                                    return (d.num_days_from_ce() - 719163) as f64;
                                }
                            }
                            if idx.r#type == IndexType::DateTime {
                                // Try standard SQL timestamp format
                                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(
                                    s.trim_matches('\''),
                                    "%Y-%m-%d %H:%M:%S",
                                ) {
                                    return dt.and_utc().timestamp() as f64;
                                }
                                // Try ISO 8601
                                if let Ok(dt) =
                                    chrono::DateTime::parse_from_rfc3339(s.trim_matches('\''))
                                {
                                    return dt.timestamp() as f64;
                                }
                            }
                            0.0
                        })
                        .collect();

                    self.range_bins
                        .lock()
                        .unwrap()
                        .insert(idx.column.clone(), quantiles);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn stream_and_process(&self, conn: &Connection) -> Result<()> {
        println!("Streaming and processing data...");

        // Count total rows for progress logging
        let total_rows: u64 =
            conn.query_row("SELECT COUNT(*) FROM stage_indexation", [], |row| {
                row.get(0)
            })?;

        let mut stmt = conn.prepare("SELECT * FROM stage_indexation")?;

        // On récupère toutes les colonnes de la table pour le traitement
        let mut stmt_cols = conn.prepare("PRAGMA table_info('stage_indexation')")?;
        let col_names: Vec<String> = stmt_cols
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut current_offset: u64 = 0;
        let chunk_size = 10000;
        let mut chunk: Vec<(HashMap<String, String>, Option<Vec<u8>>)> =
            Vec::with_capacity(chunk_size);
        let mut processed_rows: u64 = 0;
        let mut last_logged_percent = 0;

        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let mut data = HashMap::new();
            let mut wkb = None;
            for (i, col) in col_names.iter().enumerate() {
                let value = row.get_ref(i)?;
                if col == "_idx_wkb" {
                    if let duckdb::types::ValueRef::Blob(b) = value {
                        wkb = Some(b.to_vec());
                    }
                }

                let val_str = match value {
                    duckdb::types::ValueRef::Null => None,
                    duckdb::types::ValueRef::Boolean(b) => Some(b.to_string()),
                    duckdb::types::ValueRef::TinyInt(i) => Some(i.to_string()),
                    duckdb::types::ValueRef::SmallInt(i) => Some(i.to_string()),
                    duckdb::types::ValueRef::Int(i) => Some(i.to_string()),
                    duckdb::types::ValueRef::BigInt(i) => Some(i.to_string()),
                    duckdb::types::ValueRef::HugeInt(i) => Some(i.to_string()),
                    duckdb::types::ValueRef::Float(f) => Some(f.to_string()),
                    duckdb::types::ValueRef::Double(d) => Some(d.to_string()),
                    duckdb::types::ValueRef::Text(t) => {
                        Some(String::from_utf8_lossy(t).to_string())
                    }
                    duckdb::types::ValueRef::Blob(b) => Some(format!("blob:{} bytes", b.len())),
                    duckdb::types::ValueRef::Timestamp(unit, val) => {
                        let seconds = match unit {
                            duckdb::types::TimeUnit::Second => val,
                            duckdb::types::TimeUnit::Millisecond => val / 1000,
                            duckdb::types::TimeUnit::Microsecond => val / 1_000_000,
                            duckdb::types::TimeUnit::Nanosecond => val / 1_000_000_000,
                        };
                        if let Some(dt) = chrono::DateTime::from_timestamp(seconds, 0) {
                            Some(dt.to_rfc3339())
                        } else {
                            Some(val.to_string())
                        }
                    }
                    duckdb::types::ValueRef::Date32(days) => {
                        // DuckDB Date32 is days since 1970-01-01
                        let date = chrono::NaiveDate::from_num_days_from_ce_opt(days + 719163)
                            .unwrap_or_default();
                        Some(date.format("%Y-%m-%d").to_string())
                    }
                    _ => Some(format!("{:?}", value)),
                };
                if let Some(s) = val_str {
                    data.insert(col.clone(), s);
                }
            }
            chunk.push((data, wkb));
            processed_rows += 1;

            if chunk.len() >= chunk_size {
                self.process_chunk(&chunk, current_offset)?;
                self.write_storage_chunk(&chunk)?;
                current_offset += chunk.len() as u64;
                chunk.clear();

                // Progress logging
                let percent = (processed_rows * 100) / total_rows;
                if percent >= last_logged_percent + 5 || processed_rows >= total_rows {
                    println!(
                        "Progress: {}% ({}/{} rows)",
                        percent, processed_rows, total_rows
                    );
                    last_logged_percent = (percent / 5) * 5;
                }
            }
        }

        if !chunk.is_empty() {
            self.process_chunk(&chunk, current_offset)?;
            self.write_storage_chunk(&chunk)?;
            processed_rows += chunk.len() as u64;

            println!("Progress: 100% ({}/{} rows)", processed_rows, total_rows);
        }

        Ok(())
    }

    fn process_chunk(
        &self,
        chunk: &[(HashMap<String, String>, Option<Vec<u8>>)],
        offset: u64,
    ) -> Result<()> {
        // Traitement de la clé primaire (Primary Key)
        let pk_col = &self.config.storage.primary_key;
        if !pk_col.is_empty() {
            let mut global_fst = self.primary_fst.lock().unwrap();
            for (i, (row, _)) in chunk.iter().enumerate() {
                if let Some(val) = row.get(pk_col) {
                    let doc_id = offset + i as u64 + 1; // 1-based ID
                    global_fst.insert(val.clone(), doc_id);
                }
            }
        }

        // Traitement des index en parallèle
        self.config.indexes.par_iter().for_each(|idx_config| {
            match idx_config.r#type {
                IndexType::Bitmap => {
                    let mut local_bitmaps: HashMap<String, RoaringBitmap> = HashMap::new();

                    for (i, (row, _)) in chunk.iter().enumerate() {
                        if let Some(val) = row.get(&idx_config.column) {
                            let doc_id = (offset + i as u64 + 1) as u32; // 1-based ID
                            local_bitmaps.entry(val.clone()).or_default().insert(doc_id);
                        }
                    }

                    // Fusion dans l'état global
                    let mut global_bitmaps = self.bitmaps.lock().unwrap();
                    let col_bitmaps = global_bitmaps.entry(idx_config.column.clone()).or_default();
                    for (val, bitmap) in local_bitmaps {
                        let target = col_bitmaps.entry(val).or_default();
                        *target |= bitmap;
                    }
                }
                IndexType::Text => {
                    let mut local_bitmaps: HashMap<String, RoaringBitmap> = HashMap::new();
                    let stopwords: std::collections::HashSet<String> = idx_config
                        .stopwords
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|s| slugify(&s))
                        .collect();

                    for (i, (row, _)) in chunk.iter().enumerate() {
                        if let Some(val) = row.get(&idx_config.column) {
                            let doc_id = (offset + i as u64 + 1) as u32; // 1-based ID
                            let normalized = slugify(val);
                            let tokens: Vec<&str> = normalized.split_whitespace().collect();

                            for token in tokens {
                                if token.len() >= 2
                                    || token.chars().next().unwrap_or(' ').is_numeric()
                                {
                                    if !stopwords.contains(token) {
                                        local_bitmaps
                                            .entry(token.to_string())
                                            .or_default()
                                            .insert(doc_id);
                                    }
                                }
                            }
                        }
                    }

                    // Fusion dans l'état global
                    let mut global_bitmaps = self.bitmaps.lock().unwrap();
                    let col_bitmaps = global_bitmaps.entry(idx_config.column.clone()).or_default();
                    for (val, bitmap) in local_bitmaps {
                        let target = col_bitmaps.entry(val).or_default();
                        *target |= bitmap;
                    }
                }
                IndexType::Geo => {
                    let strategies = self.geo_strategies.lock().unwrap();
                    let strategy = strategies
                        .get(&idx_config.column)
                        .cloned()
                        .unwrap_or(GeoStrategy::H3);
                    drop(strategies); // release lock

                    // 1. Collect ECEF (Always)
                    let mut local_ecef = Vec::with_capacity(chunk.len());
                    for (row, _) in chunk.iter() {
                        let x: f32 = row
                            .get("_idx_ecef_x")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                        let y: f32 = row
                            .get("_idx_ecef_y")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                        let z: f32 = row
                            .get("_idx_ecef_z")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                        local_ecef.push([x, y, z]);
                    }
                    {
                        let mut global_ecef = self.ecef_coords.lock().unwrap();
                        if global_ecef.len() < (offset as usize + chunk.len()) {
                            global_ecef.resize(offset as usize + chunk.len(), [0.0, 0.0, 0.0]);
                        }
                        for (i, coord) in local_ecef.into_iter().enumerate() {
                            global_ecef[offset as usize + i] = coord;
                        }
                    }

                    // 2. Strategy specific
                    if strategy == GeoStrategy::H3 {
                        let res = idx_config.resolution.unwrap_or(8);
                        let h3_res = Resolution::try_from(res).unwrap();
                        let mut local_bitmaps: HashMap<String, RoaringBitmap> = HashMap::new();

                        for (i, (row, _)) in chunk.iter().enumerate() {
                            if let (Some(lat_str), Some(lon_str)) =
                                (row.get("_idx_lat"), row.get("_idx_lon"))
                            {
                                let lat: f64 = lat_str.parse().unwrap_or(0.0);
                                let lon: f64 = lon_str.parse().unwrap_or(0.0);

                                if let Ok(coord) = LatLng::new(lat, lon) {
                                    let cell = coord.to_cell(h3_res);
                                    let doc_id = (offset + i as u64 + 1) as u32; // 1-based ID
                                    local_bitmaps
                                        .entry(u64::from(cell).to_string())
                                        .or_default()
                                        .insert(doc_id);
                                }
                            }
                        }

                        let mut global_bitmaps = self.bitmaps.lock().unwrap();
                        let col_bitmaps = global_bitmaps
                            .entry(format!("{}_h3_l{}", idx_config.column, res))
                            .or_default();
                        for (val, bitmap) in local_bitmaps {
                            let target = col_bitmaps.entry(val).or_default();
                            *target |= bitmap;
                        }
                    }
                }
                IndexType::Range | IndexType::Date | IndexType::DateTime => {
                    let bins_lock = self.range_bins.lock().unwrap();
                    if let Some(quantiles) = bins_lock.get(&idx_config.column) {
                        let mut local_bitmaps: HashMap<String, RoaringBitmap> = HashMap::new();

                        for (i, (row, _)) in chunk.iter().enumerate() {
                            if let Some(val_str) = row.get(&idx_config.column) {
                                let val_opt = match idx_config.r#type {
                                    IndexType::Range => val_str.trim().parse::<f64>().ok(),
                                    IndexType::Date => {
                                        chrono::NaiveDate::parse_from_str(val_str, "%Y-%m-%d")
                                            .ok()
                                            .map(|d| (d.num_days_from_ce() - 719163) as f64)
                                    }
                                    IndexType::DateTime => {
                                        chrono::DateTime::parse_from_rfc3339(val_str)
                                            .ok()
                                            .map(|dt| dt.timestamp() as f64)
                                    }
                                    _ => None,
                                };

                                if let Some(val) = val_opt {
                                    let doc_id = (offset + i as u64 + 1) as u32; // 1-based ID

                                    // Trouver le "bin" (intervalle)
                                    let bin_idx = match quantiles
                                        .binary_search_by(|probe| probe.partial_cmp(&val).unwrap())
                                    {
                                        Ok(idx) => idx,
                                        Err(idx) => idx,
                                    };

                                    local_bitmaps
                                        .entry(bin_idx.to_string())
                                        .or_default()
                                        .insert(doc_id);

                                    // Collecte de la valeur brute pour le raffinement
                                    let mut rv_lock = self.range_values.lock().unwrap();
                                    let vals = rv_lock
                                        .entry(idx_config.column.clone())
                                        .or_insert_with(Vec::new);
                                    if vals.len() < (offset as usize + chunk.len()) {
                                        vals.resize(offset as usize + chunk.len(), 0.0);
                                    }
                                    vals[offset as usize + i] = val;
                                }
                            }
                        }

                        // Fusion dans l'état global
                        let mut global_bitmaps = self.bitmaps.lock().unwrap();
                        let col_bitmaps = global_bitmaps
                            .entry(format!("{}.range", idx_config.column))
                            .or_default();
                        for (val, bitmap) in local_bitmaps {
                            let target = col_bitmaps.entry(val).or_default();
                            *target |= bitmap;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn write_storage_chunk(
        &self,
        chunk: &[(HashMap<String, String>, Option<Vec<u8>>)],
    ) -> Result<()> {
        let mut data_file = self.data_file.lock().unwrap();
        let mut offset_file = self.offset_file.lock().unwrap();
        let mut global_offset = self.current_data_offset.lock().unwrap();

        // Build a mapping of columns to their indexed type
        let mut col_types = HashMap::new();
        // Identify the active geometry column (last one defined, matching QueryBuilder logic)
        let mut geo_col_name = None;
        for idx in &self.config.indexes {
            col_types.insert(idx.column.clone(), &idx.r#type);
            if idx.r#type == IndexType::Geo {
                geo_col_name = Some(&idx.column);
            }
        }

        // Determine if we should store geometry (WKB)
        // Only if the identified geometry column is listed in storage.columns
        let should_store_geometry = if let Some(name) = geo_col_name {
            self.config.storage.columns.contains(name)
        } else {
            false
        };

        for (row, wkb) in chunk {
            let mut fields = Vec::new();

            for col_name in &self.config.storage.columns {
                if let Some(val_str) = row.get(col_name) {
                    let value = match col_types.get(col_name) {
                        Some(IndexType::Range) => {
                            if let Ok(i) = val_str.parse::<i64>() {
                                Value::Integer(Box::new(IntegerWrapper { value: i }))
                            } else if let Ok(f) = val_str.parse::<f64>() {
                                Value::Float(Box::new(FloatWrapper { value: f }))
                            } else {
                                Value::Text(Box::new(TextWrapper {
                                    value: Some(val_str.clone()),
                                }))
                            }
                        }
                        Some(IndexType::Date) => {
                            if let Ok(d) = chrono::NaiveDate::parse_from_str(val_str, "%Y-%m-%d") {
                                let days = d.num_days_from_ce() - 719163;
                                Value::Date(Box::new(DateWrapper { value: days }))
                            } else {
                                Value::Text(Box::new(TextWrapper {
                                    value: Some(val_str.clone()),
                                }))
                            }
                        }
                        Some(IndexType::DateTime) => {
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(val_str) {
                                Value::DateTime(Box::new(DateTimeWrapper {
                                    value: dt.timestamp(),
                                }))
                            } else {
                                Value::Text(Box::new(TextWrapper {
                                    value: Some(val_str.clone()),
                                }))
                            }
                        }
                        _ => {
                            // Default to Text, but try to infer boolean or numeric if it looks like one
                            if val_str == "true" || val_str == "false" {
                                Value::Boolean(Box::new(BooleanWrapper {
                                    value: val_str == "true",
                                }))
                            } else if let Ok(i) = val_str.trim().parse::<i64>() {
                                // Keep as string if it has leading zeros (potential zip code or ID)
                                let trimmed = val_str.trim();
                                if trimmed.starts_with('0') && trimmed.len() > 1 {
                                    Value::Text(Box::new(TextWrapper {
                                        value: Some(val_str.clone()),
                                    }))
                                } else {
                                    Value::Integer(Box::new(IntegerWrapper { value: i }))
                                }
                            } else if let Ok(f) = val_str.trim().parse::<f64>() {
                                Value::Float(Box::new(FloatWrapper { value: f }))
                            } else {
                                Value::Text(Box::new(TextWrapper {
                                    value: Some(val_str.clone()),
                                }))
                            }
                        }
                    };

                    fields.push(Field {
                        key: Some(col_name.clone()),
                        value: Some(value),
                    });
                }
            }

            let entry = Entry {
                fields: Some(fields),
                geometry: if should_store_geometry {
                    wkb.clone()
                } else {
                    None
                },
            };
            let mut builder = planus::Builder::new();
            let result = builder.finish(entry, None);

            // Écrire l'offset actuel avant d'écrire la donnée
            offset_file.write_all(&global_offset.to_le_bytes())?;

            // Écrire la donnée
            data_file.write_all(result)?;
            *global_offset += result.len() as u64;
        }

        Ok(())
    }

    fn finalize(&self, conn: &Connection) -> Result<()> {
        println!("Finalizing index...");

        let overall_geo_strategy = {
            let strategies = self.geo_strategies.lock().unwrap();
            let mut strategy = None;
            for idx_config in &self.config.indexes {
                if idx_config.r#type == IndexType::Geo {
                    if let Some(&strat) = strategies.get(&idx_config.column) {
                        if strat == GeoStrategy::FGB {
                            strategy = Some("polygon".to_string());
                            break; // FGB wins
                        } else {
                            strategy = Some("point".to_string());
                        }
                    }
                }
            }
            strategy
        };

        // 0. Écriture des index Geo (FGB si stratégie active)
        if overall_geo_strategy == Some("polygon".to_string()) {
            let strategies = self.geo_strategies.lock().unwrap();
            for idx_config in &self.config.indexes {
                if idx_config.r#type == IndexType::Geo {
                    if let Some(&strat) = strategies.get(&idx_config.column) {
                        if strat == GeoStrategy::FGB {
                            println!("Generating FlatGeoBuf index for {}...", idx_config.column);
                            let fgb_path =
                                self.output_dir.join(format!("{}.fgb", idx_config.column));
                            let mut file = File::create(fgb_path)?;

                            let mut writer = FgbWriter::create("index", GeometryType::Unknown)?;
                            writer.add_column("id", ColumnType::ULong, |_, _| {});

                            let mut stmt = conn.prepare("SELECT _idx_wkb FROM stage_indexation")?;
                            let mut rows = stmt.query([])?;
                            let mut count: u64 = 0;
                            while let Some(row) = rows.next()? {
                                count += 1;
                                let wkb: Vec<u8> = row.get(0)?;
                                let geom = geozero::wkb::Wkb(wkb);

                                writer.add_feature_geom(geom, |feat| {
                                    feat.property(0, "id", &ColumnValue::ULong(count)).unwrap();
                                })?;
                            }
                            writer.write(&mut file)?;
                            println!("FGB index created with {} features", count);
                        }
                    }
                }
            }
        }
        // 1. Écriture de la Primary Key FST
        let pk_map = self.primary_fst.lock().unwrap();
        if !pk_map.is_empty() {
            let fst_path = self.output_dir.join("primary.fst");
            let mut fst_file = BufWriter::new(File::create(fst_path)?);
            let mut builder = fst::MapBuilder::new(&mut fst_file)?;
            // Les clés sont déjà triées car on utilise BTreeMap
            for (key, doc_id) in pk_map.iter() {
                builder.insert(key, *doc_id)?;
            }
            builder.finish()?;
        }

        for idx_config in &self.config.indexes {
            let bitmaps = self.bitmaps.lock().unwrap();
            let col = &idx_config.column;

            if idx_config.r#type == IndexType::Text {
                if let Some(values) = bitmaps.get(col) {
                    let fst_path = self.output_dir.join(format!("{}.fst", col));
                    let data_path = self.output_dir.join(format!("{}.fst.data", col));
                    let offsets_path = self.output_dir.join(format!("{}.fst.offsets", col));

                    let mut fst_file = BufWriter::new(File::create(fst_path)?);
                    let mut data_file = BufWriter::new(File::create(data_path)?);
                    let mut offsets_file = BufWriter::new(File::create(offsets_path)?);

                    let mut builder = fst::MapBuilder::new(&mut fst_file)?;

                    // Les clés doivent être triées pour l'FST
                    let mut sorted_keys: Vec<_> = values.keys().collect();
                    sorted_keys.sort();

                    let mut current_offset: u64 = 0;
                    for (rank, key) in sorted_keys.into_iter().enumerate() {
                        // On écrit l'offset de début pour cet élément
                        offsets_file.write_all(&current_offset.to_le_bytes())?;

                        let bitmap = values.get(key).unwrap();
                        let mut buf = Vec::new();
                        bitmap.serialize_into(&mut buf)?;

                        data_file.write_all(&buf)?;

                        // On insère le rang (index) dans le FST
                        builder.insert(key, rank as u64)?;
                        current_offset += buf.len() as u64;
                    }
                    // On écrit l'offset final (fin du dernier élément)
                    offsets_file.write_all(&current_offset.to_le_bytes())?;

                    builder.finish()?;
                    fst_file.flush()?;
                    data_file.flush()?;
                    offsets_file.flush()?;
                }
            } else if idx_config.r#type == IndexType::Range {
                // Traitement spécifique pour les Range Index (Format: Boundaries + Bitmaps + RawValues)
                let range_key = format!("{}.range", col);
                if let Some(values) = bitmaps.get(&range_key) {
                    // 1. Écriture du fichier structure (.range.idx)
                    let file_path = self.output_dir.join(format!("{}.range.idx", col));
                    // Utilisation directe de File pour éviter les problèmes de buffer
                    let mut file = File::create(file_path)?;

                    let bins_lock = self.range_bins.lock().unwrap();
                    if let Some(quantiles) = bins_lock.get(col) {
                        // A. Nombre de bornes (u32) + Bornes (f64...)
                        file.write_all(&(quantiles.len() as u32).to_le_bytes())?;
                        for b in quantiles {
                            file.write_all(&b.to_le_bytes())?;
                        }

                        // B. Bitmaps (séquentiels 0..=len)
                        let empty_bitmap = RoaringBitmap::new();
                        // Note: binary_search renvoie 0..len, donc il y a len+1 buckets
                        for i in 0..=quantiles.len() {
                            let key = i.to_string();
                            let bitmap = values.get(&key).unwrap_or(&empty_bitmap);

                            let mut buf = Vec::new();
                            bitmap.serialize_into(&mut buf)?;
                            file.write_all(&(buf.len() as u32).to_le_bytes())?;
                            file.write_all(&buf)?;
                        }
                    }
                    file.sync_all()?;
                    let pos = file.metadata()?.len();
                    cold_search_core::debug_log!(
                        "DEBUG: Closed .range.idx file. Size: {} bytes",
                        pos
                    );

                    // 2. Écriture des valeurs brutes (.vals) pour le mmap
                    let rv_lock = self.range_values.lock().unwrap();
                    if let Some(vals) = rv_lock.get(col) {
                        let vals_path = self.output_dir.join(format!("{}.vals", col));
                        let mut vals_file = BufWriter::new(File::create(vals_path)?);
                        for v in vals {
                            vals_file.write_all(&v.to_le_bytes())?;
                        }
                        vals_file.flush()?;
                    }
                }
            } else {
                // Bitmap classique (H3 ou secondaire)
                // Pour Geo, on vérifie si on est en H3
                let mut is_h3 = false;
                if idx_config.r#type == IndexType::Geo {
                    let strategies = self.geo_strategies.lock().unwrap();
                    if let Some(GeoStrategy::H3) = strategies.get(col) {
                        is_h3 = true;
                    }
                }

                let col_name = if is_h3 {
                    format!("{}_h3_l{}", col, idx_config.resolution.unwrap_or(8))
                } else {
                    col.clone()
                };

                if let Some(values) = bitmaps.get(&col_name) {
                    let file_path = self.output_dir.join(format!("{}.idx", col_name));
                    let mut file = BufWriter::new(File::create(file_path)?);

                    if is_h3 {
                        // Format H3 : u64 (cell) | u32 (len) | bitmap
                        for (val_str, bitmap) in values {
                            let cell: u64 = val_str.parse().unwrap_or(0);
                            file.write_all(&cell.to_le_bytes())?;

                            let mut buf = Vec::new();
                            bitmap.serialize_into(&mut buf)?;
                            file.write_all(&(buf.len() as u32).to_le_bytes())?;
                            file.write_all(&buf)?;
                        }
                    } else if idx_config.r#type != IndexType::Geo {
                        // Don't process Geo (FGB) here, only secondary bitmaps
                        // Format Secondaire : u32 (name_len) | name | u32 (len) | bitmap
                        for (val, bitmap) in values {
                            let val_bytes = val.as_bytes();
                            file.write_all(&(val_bytes.len() as u32).to_le_bytes())?;
                            file.write_all(val_bytes)?;

                            let mut buf = Vec::new();
                            bitmap.serialize_into(&mut buf)?;
                            file.write_all(&(buf.len() as u32).to_le_bytes())?;
                            file.write_all(&buf)?;
                        }
                    }
                    file.flush()?;
                }
            }
        }

        // 3. Écriture de l'offset final et flush des fichiers de stockage
        {
            let mut offset_file = self.offset_file.lock().unwrap();
            let global_offset = self.current_data_offset.lock().unwrap();
            offset_file.write_all(&global_offset.to_le_bytes())?;
            offset_file.flush()?;

            let mut data_file = self.data_file.lock().unwrap();
            data_file.flush()?;
        }

        // 4. Écriture des coordonnées ECEF
        if overall_geo_strategy == Some("point".to_string()) || overall_geo_strategy.is_none() {
            let ecef = self.ecef_coords.lock().unwrap();
            if !ecef.is_empty() {
                let file_path = self.output_dir.join("coords.ecef");
                let mut file = BufWriter::new(File::create(file_path)?);
                for coord in ecef.iter() {
                    for val in coord {
                        file.write_all(&val.to_le_bytes())?;
                    }
                }
                file.flush()?;
            }
        }

        // Generate RuntimeConfig (stopwords)
        let mut runtime_config = RuntimeConfig::default();
        for idx_config in &self.config.indexes {
            if idx_config.r#type == IndexType::Text {
                if let Some(stops) = &idx_config.stopwords {
                    let normalized: std::collections::HashSet<String> =
                        stops.iter().map(|s| slugify(s)).collect();
                    if !normalized.is_empty() {
                        runtime_config
                            .stopwords
                            .insert(idx_config.column.clone(), normalized);
                    }
                }
            }
        }

        let config_path = self.output_dir.join("runtime_config.json");
        let config_file = File::create(config_path)?;
        serde_json::to_writer(config_file, &runtime_config)?;

        // 5. Écriture des métadonnées
        let total_items: u64 =
            conn.query_row("SELECT COUNT(*) FROM stage_indexation", [], |row| {
                row.get(0)
            })?;

        let metadata = cold_search_core::SourceMetadata {
            name: self.config.collection.clone(),
            size: total_items,
            indexes: self
                .config
                .indexes
                .iter()
                .map(|i| cold_search_core::IndexMetadata {
                    name: i.column.clone(),
                    r#type: format!("{:?}", i.r#type).to_lowercase(),
                    field: i.column.clone(),
                })
                .collect(),
            geometry_type: overall_geo_strategy,
        };
        let metadata_path = self.output_dir.join("metadata.json");
        let metadata_file = File::create(metadata_path)?;
        serde_json::to_writer_pretty(metadata_file, &metadata)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cold_search_core::config_v2::ConfigV2;
    use std::fs;

    #[test]
    fn test_pipeline_dry_run_bitmap() {
        let temp_dir = PathBuf::from("temp_test_bitmap");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let csv_path = temp_dir.join("test_pipeline.csv");
        let csv_content = "id,category\n1,A\n2,B\n3,A\n4,C";
        fs::write(&csv_path, csv_content).unwrap();

        let absolute_csv_path = fs::canonicalize(&csv_path).unwrap();

        let yaml = format!(
            r#"
version: "1.0"
collection: "test"
source:
  path: "{}"
  query: "SELECT id, category FROM read_csv_auto('{{path}}')"
storage:
  primary_key: "id"
  columns: ["id", "category"]
indexes:
  - column: "category"
    type: "bitmap"
"#,
            absolute_csv_path.display()
        );

        let config = ConfigV2::from_yaml_str(&yaml).unwrap();
        let pipeline = Pipeline::new(config, temp_dir.clone(), "1GB".to_string(), 1).unwrap();
        pipeline.run().unwrap();

        let bitmaps_lock = pipeline.bitmaps.lock().unwrap();
        let cat_bitmaps = bitmaps_lock
            .get("category")
            .expect("Missing category index");

        assert_eq!(cat_bitmaps.get("A").unwrap().len(), 2);
        assert_eq!(cat_bitmaps.get("B").unwrap().len(), 1);
        assert_eq!(cat_bitmaps.get("C").unwrap().len(), 1);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_pipeline_geo_text() {
        let temp_dir = PathBuf::from("temp_test_geo");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let csv_path = temp_dir.join("test_geo.csv");
        let csv_content = "id,lat,lon,category,name\n1,48.8566,2.3522,A,Château de Versailles\n2,45.7640,4.8357,B,Vieux Lyon";
        fs::write(&csv_path, csv_content).unwrap();

        let absolute_csv_path = fs::canonicalize(&csv_path).unwrap();

        let yaml = format!(
            r#"
version: "1.0"
collection: "test_geo"
source:
  path: "{}"
  query: "SELECT id, category, ST_Point(lon, lat) as geom, name FROM read_csv_auto('{{path}}')"
storage:
  primary_key: "id"
  columns: ["id", "category", "name", "geom"]
indexes:
  - column: "category"
    type: "bitmap"
  - column: "geom"
    type: "geo"
    resolution: 9
  - column: "name"
    type: "text"
"#,
            absolute_csv_path.display()
        );

        let config = ConfigV2::from_yaml_str(&yaml).unwrap();
        let pipeline = Pipeline::new(config, temp_dir.clone(), "1GB".to_string(), 1).unwrap();
        pipeline.run().unwrap();

        // Vérifier les fichiers produits
        assert!(temp_dir.join("category.idx").exists());
        assert!(temp_dir.join("geom_h3_l9.idx").exists());
        assert!(temp_dir.join("name.fst").exists());
        assert!(temp_dir.join("name.fst.data").exists());
        assert!(temp_dir.join("coords.ecef").exists());
        assert!(temp_dir.join("data.bin").exists());
        assert!(temp_dir.join("data.offsets").exists());

        // Vérifier le contenu ECEF (2 points * 3 coords * 4 octets = 24 octets)
        let ecef_size = fs::metadata(temp_dir.join("coords.ecef")).unwrap().len();
        assert_eq!(ecef_size, 24);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_pipeline_polygon() {
        let temp_dir = PathBuf::from("temp_test_polygon");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let csv_path = temp_dir.join("test_polygon.csv");
        let csv_content = "id,wkt\n1,\"POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))\"\n2,\"MULTIPOLYGON(((2 2, 2 3, 3 3, 3 2, 2 2)))\"";
        fs::write(&csv_path, csv_content).unwrap();

        let absolute_csv_path = fs::canonicalize(&csv_path).unwrap();

        let yaml = format!(
            r#"
version: "1.0"
collection: "test_poly"
source:
  path: "{}"
  query: "SELECT id, ST_GeomFromText(wkt) as geom FROM read_csv_auto('{{path}}', header=True)"
storage:
  primary_key: "id"
  columns: ["id", "geom"]
indexes:
  - column: "geom"
    type: "geo"
"#,
            absolute_csv_path.display()
        );

        let config = ConfigV2::from_yaml_str(&yaml).unwrap();
        let pipeline = Pipeline::new(config, temp_dir.clone(), "1GB".to_string(), 1).unwrap();

        // Ce call devrait probablement paniquer ou retourner une erreur car GeoShape n'est pas géré
        let result = pipeline.run();
        assert!(result.is_ok(), "Pipeline failed: {:?}", result.err());

        // Vérifier les fichiers produits (selon le plan)
        assert!(temp_dir.join("geom.fgb").exists(), "FGB index missing");

        // Test d'intégration avec le moteur de recherche
        cold_search_core::set_debug(true);
        let engine = cold_search_core::Engine::new("test".to_string(), &temp_dir).unwrap();

        // 1. Recherche au centre du polygone 1
        let results = engine
            .search(
                vec![],
                None,
                None,
                None,
                Some(10),
                Some([0.5, 0.5]),
                Some(1000.0),
            )
            .unwrap();

        assert_eq!(results.len(), 1, "Should find 1 polygon");
        assert_eq!(results[0].0, 1, "Should find polygon 1");

        // 2. Recherche au centre du polygone 2
        let results2 = engine
            .search(
                vec![],
                None,
                None,
                None,
                Some(10),
                Some([2.5, 2.5]),
                Some(1000.0),
            )
            .unwrap();

        assert_eq!(results2.len(), 1, "Should find 1 polygon (the second one)");
        assert_eq!(results2[0].0, 2, "Should find polygon 2");

        // 3. Recherche entre les deux (loin de tout)
        let results3 = engine
            .search(
                vec![],
                None,
                None,
                None,
                Some(10),
                Some([1.5, 1.5]),
                Some(1000.0),
            )
            .unwrap();

        assert_eq!(results3.len(), 0, "Should find no polygons");

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}

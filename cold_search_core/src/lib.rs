pub mod config_v2;
pub mod filter;
pub mod schema_generated;

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

pub static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_debug() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::is_debug() {
            println!($($arg)*);
        }
    };
}

use deunicode::deunicode;
use fst::{Automaton, IntoStreamer, Map, Streamer};
use h3o::{LatLng, Resolution};
use memmap2::Mmap;
use ordered_float::OrderedFloat;
use planus::ReadAsRoot;
use roaring::RoaringBitmap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::convert::TryInto;
use std::fs::File;
use std::path::Path;
use std::time::Instant;

use flatgeobuf::FallibleStreamingIterator;
use flatgeobuf::FeatureProperties;
use geo::ClosestPoint;
use geo::Contains;
use geo::Distance;
use geo::Haversine;
use geozero::wkb::Wkb;
use geozero::ToGeo;

/// Convert text to a normalized slug: remove accents, lowercase, keep only ASCII alphanumeric.
pub fn slugify(text: &str) -> String {
    let mut s = deunicode(text).to_lowercase();
    s.retain(|c| c.is_ascii_alphanumeric() || c == ' ');
    s
}

/// Runtime configuration for the index, currently holding stopwords per field.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    /// Stopwords per field: field name -> set of stopword strings
    pub stopwords: HashMap<String, HashSet<String>>,
}

/// Stores document metadata via memory-mapped offset and data files.
pub struct MetadataStore {
    /// Memory-mapped offsets file (8-byte offsets per document)
    offsets_mmap: Mmap,
    /// Memory-mapped data file (serialized document entries)
    data_mmap: Mmap,
    /// Bitmap of all document IDs (1..num_docs)
    pub all_ids: RoaringBitmap,
}

impl MetadataStore {
    /// Creates a new metadata store by memory‑mapping the offset and data files.
    ///
    /// The directory must contain `data.offsets` (8‑byte offsets) and `data.bin` (serialized entries).
    /// The number of documents is derived from the offsets file length.
    pub fn new<P: AsRef<Path>>(directory: P) -> Result<Self> {
        let dir = directory.as_ref();
        let offsets_file = File::open(dir.join("data.offsets"))?;
        let offsets_mmap = unsafe { Mmap::map(&offsets_file)? };
        let data_file = File::open(dir.join("data.bin"))?;
        let data_mmap = unsafe { Mmap::map(&data_file)? };

        let num_docs = (offsets_mmap.len() / 8).saturating_sub(1) as u32;
        let all_ids = RoaringBitmap::from_iter(1..=num_docs);

        Ok(Self {
            offsets_mmap,
            data_mmap,
            all_ids,
        })
    }

    pub fn get_item(&self, id: u64) -> Option<Item<'_>> {
        if id == 0 {
            return None;
        }
        let idx = (id - 1) as usize;
        let offsets_len = self.offsets_mmap.len() / 8;
        if idx + 1 >= offsets_len {
            return None;
        }
        let start_bytes = &self.offsets_mmap[idx * 8..(idx + 1) * 8];
        let end_bytes = &self.offsets_mmap[(idx + 1) * 8..(idx + 2) * 8];
        let start = u64::from_le_bytes(start_bytes.try_into().unwrap()) as usize;
        let end = u64::from_le_bytes(end_bytes.try_into().unwrap()) as usize;
        if start <= end && end <= self.data_mmap.len() {
            Some(Item {
                data: &self.data_mmap[start..end],
                id,
            })
        } else {
            None
        }
    }

    pub fn buffer_len(&self) -> usize {
        self.data_mmap.len()
    }
}

/// Index for range queries on numeric fields.
/// Boundaries define bin edges, bitmaps store document IDs per bin.
pub struct RangeIndex {
    /// Sorted bin boundaries (length = number of bins - 1)
    pub boundaries: Vec<f64>,
    /// Bitmaps for each bin (length = number of bins)
    pub bitmaps: Vec<RoaringBitmap>,
    /// Optional memory-mapped file of raw values for refinement
    pub values_mmap: Option<Mmap>,
}

#[derive(Debug, Clone)]
/// Query for range filters, with optional min/max bounds as strings.
pub struct RangeQuery {
    /// Minimum bound (inclusive), parsed according to field type
    pub min: Option<String>,
    /// Maximum bound (inclusive), parsed according to field type
    pub max: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Metadata for a single index (field) in a source.
pub struct IndexMetadata {
    /// Index name (e.g., "h3_l5", "secondary_name")
    pub name: String,
    /// Data type ("Text", "Integer", "Float", "Date", "DateTime", etc.)
    pub r#type: String,
    /// Field name in the schema
    pub field: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Metadata for a data source (dataset).
pub struct SourceMetadata {
    /// Source name (e.g., "pois", "cadastre")
    pub name: String,
    /// Approximate number of documents
    pub size: u64,
    /// List of indexes available for this source
    pub indexes: Vec<IndexMetadata>,
    /// Geometry type if the source contains spatial data ("Polygon", "Point", etc.)
    pub geometry_type: Option<String>,
}

/// In-memory representation of all indexes for a source.
pub struct IndexStore {
    /// Source metadata if available
    pub metadata: Option<SourceMetadata>,
    /// Runtime configuration (stopwords, etc.)
    pub runtime_config: RuntimeConfig,
    /// H3 spatial indexes keyed by resolution (u8) -> cell ID -> bitmap
    pub h3: HashMap<u8, HashMap<u64, RoaringBitmap>>,
    /// Secondary indexes for exact‑match fields: field name -> value -> bitmap
    pub secondary: HashMap<String, HashMap<String, RoaringBitmap>>,
    /// FST‑based text indexes: field name -> (FST mmap, data mmap, offsets mmap)
    pub fst_mmaps: HashMap<String, (Mmap, Mmap, Option<Mmap>)>,
    /// Memory‑mapped ECEF coordinates (12 bytes per document: x, y, z as f32)
    pub coords_mmap: Option<Mmap>,
    /// Range indexes for numeric/date fields: field name -> RangeIndex
    pub ranges: HashMap<String, RangeIndex>,
    /// Primary key FST (key -> doc ID) if available
    pub primary_mmap: Option<Mmap>,
    /// FlatGeoBuf geometry indexes: field name -> memory‑mapped FGB file
    pub fgb: HashMap<String, Mmap>,
}

impl IndexStore {
    /// Loads all indexes from a directory.
    ///
    /// The directory must contain the index files generated by the indexer:
    /// - `runtime_config.json` (optional)
    /// - H3 index files (`h3_l5.idx`, `h3_l7.idx`, …)
    /// - Secondary index files (`<field>.idx`)
    /// - FST text index files (`<field>.fst`, `<field>.fst.data`, `<field>.fst.offsets`)
    /// - Range index files (`<field>.range.idx`, `<field>.vals`)
    /// - Coordinate file (`coords.ecef`)
    /// - Primary key FST (`primary.fst`)
    /// - FlatGeoBuf files (`<field>.fgb`)
    /// - Metadata (`metadata.json`)
    ///
    /// Returns an `IndexStore` with memory‑mapped files ready for querying.
    pub fn load<P: AsRef<Path>>(directory: P) -> Result<Self> {
        let dir = directory.as_ref();
        let runtime_config = if let Ok(file) = File::open(dir.join("runtime_config.json")) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            RuntimeConfig::default()
        };

        fn load_h3_index(
            dir: &Path,
            filename: &str,
        ) -> Result<Option<HashMap<u64, RoaringBitmap>>> {
            let path = dir.join(filename);
            if !path.exists() {
                return Ok(None);
            }
            let mut map = HashMap::new();
            let mut file = File::open(path)?;
            use std::io::Read;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let mut cursor = std::io::Cursor::new(buf);
            while cursor.position() < cursor.get_ref().len() as u64 {
                let mut cell_buf = [0u8; 8];
                cursor.read_exact(&mut cell_buf)?;
                let cell = u64::from_le_bytes(cell_buf);
                let mut len_buf = [0u8; 4];
                cursor.read_exact(&mut len_buf)?;
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut bitmap_buf = vec![0u8; len];
                cursor.read_exact(&mut bitmap_buf)?;
                let bitmap = RoaringBitmap::deserialize_from(&bitmap_buf[..])?;
                map.insert(cell, bitmap);
            }
            Ok(Some(map))
        }

        let mut h3: HashMap<u8, HashMap<u64, RoaringBitmap>> = HashMap::new();
        if let Ok(Some(idx)) = load_h3_index(dir, "h3_l5.idx") {
            h3.insert(5, idx);
        }
        if let Ok(Some(idx)) = load_h3_index(dir, "h3_l7.idx") {
            h3.insert(7, idx);
        }
        if let Ok(Some(idx)) = load_h3_index(dir, "h3_l9.idx") {
            h3.insert(9, idx);
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name();
                let name = filename.to_string_lossy();
                if name.contains("_h3_l") && name.ends_with(".idx") {
                    if let Some(res_str) =
                        name.split("_h3_l").last().and_then(|s| s.split('.').next())
                    {
                        if let Ok(res) = res_str.parse::<u8>() {
                            if let Ok(Some(idx)) = load_h3_index(dir, &name) {
                                debug_log!("DEBUG: Loaded H3 index L{} from {}", res, name);
                                h3.entry(res).or_default().extend(idx);
                            }
                        }
                    }
                }
            }
        }

        let mut secondary = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.ends_with(".idx")
                        && !filename.starts_with("h3_")
                        && !filename.contains("_h3_l")
                        && !filename.ends_with(".range.idx")
                    {
                        let field = filename.trim_end_matches(".idx").to_string();
                        if let Ok(mut file) = File::open(&path) {
                            use std::io::Read;
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf)?;
                            let mut cursor = std::io::Cursor::new(buf);
                            let mut field_map = HashMap::new();
                            while cursor.position() < cursor.get_ref().len() as u64 {
                                let mut name_len_buf = [0u8; 4];
                                cursor.read_exact(&mut name_len_buf)?;
                                let name_len = u32::from_le_bytes(name_len_buf) as usize;
                                let mut name_buf = vec![0u8; name_len];
                                cursor.read_exact(&mut name_buf)?;
                                let name = String::from_utf8(name_buf)?;
                                let mut len_buf = [0u8; 4];
                                cursor.read_exact(&mut len_buf)?;
                                let len = u32::from_le_bytes(len_buf) as usize;
                                let mut bitmap_buf = vec![0u8; len];
                                cursor.read_exact(&mut bitmap_buf)?;
                                let bitmap = RoaringBitmap::deserialize_from(&bitmap_buf[..])?;
                                field_map.insert(name, bitmap);
                            }
                            secondary.insert(field, field_map);
                        }
                    }
                }
            }
        }

        let mut fst_mmaps = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.ends_with(".fst") && filename != "primary.fst" {
                        let field = filename.trim_end_matches(".fst").to_string();
                        let data_path = dir.join(format!("{}.fst.data", field));
                        let offsets_path = dir.join(format!("{}.fst.offsets", field));
                        if let (Ok(fst_file), Ok(data_file)) =
                            (File::open(&path), File::open(data_path))
                        {
                            let fst_mmap = unsafe { Mmap::map(&fst_file)? };
                            let data_mmap = unsafe { Mmap::map(&data_file)? };
                            let offsets_mmap = if let Ok(offsets_file) = File::open(offsets_path) {
                                Some(unsafe { Mmap::map(&offsets_file)? })
                            } else {
                                None
                            };
                            fst_mmaps.insert(field, (fst_mmap, data_mmap, offsets_mmap));
                        }
                    }
                }
            }
        }

        let coords_mmap = if let Ok(file) = File::open(dir.join("coords.ecef")) {
            Some(unsafe { Mmap::map(&file)? })
        } else {
            None
        };
        let metadata = if let Ok(file) = File::open(dir.join("metadata.json")) {
            serde_json::from_reader(file).ok()
        } else {
            None
        };

        let mut ranges = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.ends_with(".range.idx") {
                        let field = filename.trim_end_matches(".range.idx").to_string();
                        if let Ok(mut file) = File::open(&path) {
                            use std::io::Read;
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf)?;
                            let mut cursor = std::io::Cursor::new(buf);
                            let mut b_len_buf = [0u8; 4];
                            cursor.read_exact(&mut b_len_buf)?;
                            let b_len = u32::from_le_bytes(b_len_buf) as usize;

                            let mut boundaries = Vec::with_capacity(b_len);
                            for _ in 0..b_len {
                                let mut b_buf = [0u8; 8];
                                cursor.read_exact(&mut b_buf)?;
                                boundaries.push(f64::from_le_bytes(b_buf));
                            }

                            let mut bitmaps = Vec::with_capacity(b_len);
                            while cursor.position() < cursor.get_ref().len() as u64 {
                                let mut len_buf = [0u8; 4];
                                cursor.read_exact(&mut len_buf)?;
                                let len = u32::from_le_bytes(len_buf) as usize;
                                let mut bitmap_buf = vec![0u8; len];
                                cursor.read_exact(&mut bitmap_buf)?;
                                bitmaps.push(RoaringBitmap::deserialize_from(&bitmap_buf[..])?);
                            }
                            let vals_path = dir.join(format!("{}.vals", field));
                            let values_mmap = if let Ok(v_file) = File::open(vals_path) {
                                Some(unsafe { Mmap::map(&v_file)? })
                            } else {
                                None
                            };
                            ranges.insert(
                                field,
                                RangeIndex {
                                    boundaries,
                                    bitmaps,
                                    values_mmap,
                                },
                            );
                        }
                    }
                }
            }
        }

        let mut fgb = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.ends_with(".fgb") {
                        let field = filename.trim_end_matches(".fgb").to_string();
                        if let Ok(file) = File::open(&path) {
                            let mmap = unsafe { Mmap::map(&file)? };
                            fgb.insert(field, mmap);
                        }
                    }
                }
            }
        }

        let primary_mmap = if let Ok(file) = File::open(dir.join("primary.fst")) {
            Some(unsafe { Mmap::map(&file)? })
        } else {
            None
        };

        Ok(IndexStore {
            metadata,
            runtime_config,
            h3,
            secondary,
            fst_mmaps,
            coords_mmap,
            ranges,
            primary_mmap,
            fgb,
        })
    }

    pub fn info(&self) -> Vec<String> {
        let mut info = Vec::new();
        if !self.h3.is_empty() {
            let res: Vec<_> = self.h3.keys().collect();
            info.push(format!("H3 Resolutions: {:?}", res));
        }
        if !self.secondary.is_empty() {
            let fields: Vec<_> = self.secondary.keys().collect();
            info.push(format!("Secondary: {:?}", fields));
        }
        if !self.fst_mmaps.is_empty() {
            let fields: Vec<_> = self.fst_mmaps.keys().collect();
            info.push(format!("FST: {:?}", fields));
        }
        if !self.ranges.is_empty() {
            let fields: Vec<_> = self.ranges.keys().collect();
            info.push(format!("Range: {:?}", fields));
        }
        info
    }

    pub fn get_id_by_primary_key(&self, key: &str) -> Option<u64> {
        if let Some(mmap) = &self.primary_mmap {
            let map = Map::new(mmap).ok()?;
            map.get(key)
        } else {
            None
        }
    }

    pub fn get_ecef(&self, id: u64) -> Option<[f32; 3]> {
        if let Some(mmap) = &self.coords_mmap {
            let idx = (id - 1) as usize;
            let start = idx * 12;
            if start + 12 <= mmap.len() {
                let x = f32::from_le_bytes(mmap[start..start + 4].try_into().unwrap());
                let y = f32::from_le_bytes(mmap[start + 4..start + 8].try_into().unwrap());
                let z = f32::from_le_bytes(mmap[start + 8..start + 12].try_into().unwrap());
                return Some([x, y, z]);
            }
        }
        None
    }
}

/// Main search engine for a source, combining metadata and indexes.
pub struct Engine {
    /// Source name (identifier)
    pub name: String,
    /// Metadata store for document retrieval
    pub metadata: MetadataStore,
    /// Index store for query processing
    pub index: IndexStore,
}

impl Engine {
    pub fn new<P: AsRef<Path>>(name: String, directory: P) -> Result<Self> {
        let dir = directory.as_ref();
        let metadata = MetadataStore::new(dir)?;
        let index = IndexStore::load(dir)?;
        Ok(Self {
            name,
            metadata,
            index,
        })
    }

    pub fn info(&self) -> Vec<String> {
        self.index.info()
    }
    pub fn get(&self, id: u64) -> Option<Item<'_>> {
        self.metadata.get_item(id)
    }
    pub fn get_by_key(&self, key: &str) -> Option<Item<'_>> {
        let id = self.index.get_id_by_primary_key(key)?;
        self.get(id)
    }
    pub fn get_ecef(&self, id: u64) -> Option<[f32; 3]> {
        self.index.get_ecef(id)
    }

    pub fn bbox_h3_cells(resolution: Resolution, bbox: [f64; 4]) -> anyhow::Result<Vec<u64>> {
        let [min_lon, min_lat, max_lon, max_lat] = bbox;
        let mut cells = std::collections::HashSet::new();
        let steps = 20;
        for i in 0..=steps {
            for j in 0..=steps {
                let lat = min_lat + (max_lat - min_lat) * (i as f64 / steps as f64);
                let lon = min_lon + (max_lon - min_lon) * (j as f64 / steps as f64);
                let ll = LatLng::new(lat, lon)?;
                cells.insert(u64::from(ll.to_cell(resolution)));
            }
        }
        Ok(cells.into_iter().collect())
    }

    fn get_all_ids(&self) -> RoaringBitmap {
        self.metadata.all_ids.clone()
    }

    fn parse_range_value(&self, field: &str, value: &str) -> Option<f64> {
        let field_type = self
            .index
            .metadata
            .as_ref()
            .and_then(|m| m.indexes.iter().find(|i| i.field == field))
            .map(|i| i.r#type.as_str())
            .unwrap_or("Float");

        match field_type {
            "Date" => {
                use chrono::NaiveDate;
                NaiveDate::parse_from_str(value, "%Y-%m-%d").ok().map(|d| {
                    let base = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                    d.signed_duration_since(base).num_days() as f64
                })
            }
            "DateTime" => {
                use chrono::DateTime;
                DateTime::parse_from_rfc3339(value)
                    .map(|dt| dt.timestamp() as f64)
                    .ok()
                    .or_else(|| {
                        // Fallback for non-Z dates if needed, or simple YYYY-MM-DD HH:MM:SS
                        use chrono::NaiveDateTime;
                        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                            .ok()
                            .map(|dt| dt.and_utc().timestamp() as f64)
                    })
            }
            _ => value.parse::<f64>().ok(),
        }
    }

    /// Performs a search with filters, spatial bounds, text query, and grouping.
    ///
    /// # Parameters
    /// - `filters`: equality, inequality, inclusion, range filters
    /// - `bbox`: optional bounding box `[min_lon, min_lat, max_lon, max_lat]`
    /// - `text`: optional full‑text query (AND semantics across words)
    /// - `group_by`: optional field name for per‑group top‑N results
    /// - `limit`: maximum number of results to return
    /// - `center`: optional center point `[lat, lon]` for distance sorting / radius filtering
    /// - `radius_m`: optional radius in meters around `center`
    ///
    /// # Returns
    /// A vector of `(document_id, distance)` pairs sorted by distance if a center is provided,
    /// otherwise by document order. Distance is zero when no center is given.
    ///
    /// # Algorithm
    /// 1. Start with all IDs or empty bitmap depending on presence of text/filters.
    /// 2. Apply text search via FST indexes (prefix matching, stopword filtering).
    /// 3. Apply each filter, intersecting bitmaps.
    /// 4. If a center is given, adaptively expand H3 rings until enough candidates are found.
    /// 5. Perform spatial refinement using FlatGeoBuf (if available) or ECEF coordinates.
    /// 6. Apply grouping (if requested) and limit.
    /// 7. Return sorted results.
    pub fn search(
        &self,
        filters: Vec<filter::Filter>,
        bbox: Option<[f64; 4]>,
        text: Option<&str>,
        group_by: Option<&str>,
        limit: Option<usize>,
        center: Option<[f64; 2]>,
        radius_m: Option<f64>,
    ) -> Result<Vec<(u64, f64)>> {
        let start_time = Instant::now();
        debug_log!(
            "[{:.3?}] Search started with {} filters, text: {:?}, group_by: {:?}",
            start_time.elapsed(),
            filters.len(),
            text,
            group_by
        );

        let mut candidates = if text.is_none() && filters.is_empty() && bbox.is_none() {
            self.get_all_ids()
        } else {
            RoaringBitmap::new()
        };

        if let Some(q) = text {
            let mut results_opt: Option<RoaringBitmap> = None;
            for word in q.split_whitespace() {
                let mut word_results = RoaringBitmap::new();
                let mut word_is_stopword_everywhere = true;
                let slug = slugify(word);

                for (field, (fst_mmap, data_mmap, _)) in &self.index.fst_mmaps {
                    if let Some(stopwords) = self.index.runtime_config.stopwords.get(field) {
                        if stopwords.contains(&slug) {
                            continue;
                        }
                    }
                    word_is_stopword_everywhere = false;

                    let fst = fst::Map::new(fst_mmap)?;
                    let mut stream = fst
                        .search(fst::automaton::Str::new(&slug).starts_with())
                        .into_stream();
                    while let Some((_, offset_val)) = stream.next() {
                        let offset = (offset_val >> 32) as usize;
                        let len = (offset_val & 0xFFFFFFFF) as usize;
                        if offset + len <= data_mmap.len() {
                            if let Ok(bitmap) =
                                RoaringBitmap::deserialize_from(&data_mmap[offset..offset + len])
                            {
                                word_results |= bitmap;
                            }
                        }
                    }
                }
                if word_is_stopword_everywhere {
                    continue;
                }
                if let Some(current) = results_opt {
                    results_opt = Some(current & word_results);
                } else {
                    results_opt = Some(word_results);
                }
            }
            if let Some(text_results) = results_opt {
                candidates = text_results;
                debug_log!(
                    "[{:.3?}] Text search done, candidates: {}",
                    start_time.elapsed(),
                    candidates.len()
                );
            }
        }

        for filter in &filters {
            let filter_bitmap = match filter {
                filter::Filter::Equal(field, val) => {
                    if let Some(field_map) = self.index.secondary.get(field.as_str()) {
                        field_map.get(val.as_str()).cloned().unwrap_or_default()
                    } else {
                        RoaringBitmap::new()
                    }
                }
                filter::Filter::NotEqual(field, val) => {
                    let mut all = self.get_all_ids();
                    if let Some(field_map) = self.index.secondary.get(field.as_str()) {
                        if let Some(bitmap) = field_map.get(val.as_str()) {
                            all -= bitmap;
                        }
                    }
                    all
                }
                filter::Filter::In(field, vals) => {
                    let mut res = RoaringBitmap::new();
                    if let Some(field_map) = self.index.secondary.get(field.as_str()) {
                        for val in vals {
                            if let Some(bitmap) = field_map.get(val.as_str()) {
                                res |= bitmap;
                            }
                        }
                    }
                    res
                }
                filter::Filter::NotIn(field, vals) => {
                    let mut all = self.get_all_ids();
                    if let Some(field_map) = self.index.secondary.get(field.as_str()) {
                        for val in vals {
                            if let Some(bitmap) = field_map.get(val.as_str()) {
                                all -= bitmap;
                            }
                        }
                    }
                    all
                }
                filter::Filter::Range(field, query) => {
                    if let Some(range_index) = self.index.ranges.get(field.as_str()) {
                        debug_log!(
                            "DEBUG: Range boundaries for {}: {:?}",
                            field,
                            range_index.boundaries
                        );
                        let mut range_bitmap = RoaringBitmap::new();
                        let num_bins = range_index.boundaries.len() + 1;
                        let q_min = query
                            .min
                            .as_ref()
                            .and_then(|v| self.parse_range_value(field, v));
                        let q_max = query
                            .max
                            .as_ref()
                            .and_then(|v| self.parse_range_value(field, v));

                        for i in 0..num_bins {
                            let lower = if i == 0 {
                                f64::NEG_INFINITY
                            } else {
                                range_index.boundaries[i - 1]
                            };
                            let upper = if i == range_index.boundaries.len() {
                                f64::INFINITY
                            } else {
                                range_index.boundaries[i]
                            };

                            let overlaps = match (q_min, q_max) {
                                (Some(min), Some(max)) => min <= upper && max >= lower,
                                (Some(min), None) => min <= upper,
                                (None, Some(max)) => max >= lower,
                                (None, None) => true,
                            };

                            if overlaps {
                                debug_log!(
                                    "DEBUG: Bin {} [{}, {}] overlaps query {:?}",
                                    i,
                                    lower,
                                    upper,
                                    query
                                );
                                if let Some(bitmap) = range_index.bitmaps.get(i) {
                                    debug_log!(
                                        "DEBUG: Adding {} candidates from bin {}",
                                        bitmap.len(),
                                        i
                                    );
                                    range_bitmap |= bitmap;
                                }
                            }
                        }
                        debug_log!("DEBUG: Pre-refinement candidates: {}", range_bitmap.len());

                        if let Some(mmap) = &range_index.values_mmap {
                            let mut filtered = RoaringBitmap::new();
                            let mut kept_count = 0;
                            let mut debug_log_count = 0;

                            for id in range_bitmap.iter() {
                                let idx = (id - 1) as usize;
                                let start = idx * 8;
                                if start + 8 <= mmap.len() {
                                    let val = f64::from_le_bytes(
                                        mmap[start..start + 8].try_into().unwrap(),
                                    );
                                    let matches = match (q_min, q_max) {
                                        (Some(min), Some(max)) => val >= min && val <= max,
                                        (Some(min), None) => val >= min,
                                        (None, Some(max)) => val <= max,
                                        (None, None) => true,
                                    };

                                    if debug_log_count < 5 {
                                        debug_log!(
                                            "DEBUG: Refinement id={} val={} matches={}",
                                            id,
                                            val,
                                            matches
                                        );
                                        debug_log_count += 1;
                                    }
                                    if matches {
                                        filtered.insert(id);
                                        kept_count += 1;
                                    }
                                }
                            }
                            debug_log!("DEBUG: Post-refinement kept: {}", kept_count);
                            range_bitmap = filtered;
                        }
                        range_bitmap
                    } else {
                        debug_log!(
                            "[{:.3?}] WARNING: Range index for field '{}' not found!",
                            start_time.elapsed(),
                            field
                        );
                        RoaringBitmap::new()
                    }
                }
            };

            if candidates.is_empty() && text.is_none() {
                candidates = filter_bitmap;
            } else {
                candidates &= filter_bitmap;
            }
            debug_log!(
                "[{:.3?}] Filter {:?} applied, candidates: {}",
                start_time.elapsed(),
                filter,
                candidates.len()
            );
        }

        if let Some(_) = bbox {
            if candidates.is_empty() {
                debug_log!(
                    "[{:.3?}] Optimization: 0 candidates after filters, skipping H3",
                    start_time.elapsed()
                );
                return Ok(Vec::new());
            }
        }

        if let (Some([lat, lon]), Some(limit_val)) = (center, limit) {
            if bbox.is_none() && !self.index.h3.is_empty() {
                let adaptive_start = Instant::now();
                let mut available_res: Vec<u8> = self.index.h3.keys().cloned().collect();
                available_res.sort_by(|a, b| b.cmp(a));
                if let Some(&res_u8) = available_res.first() {
                    let res = Resolution::try_from(res_u8).unwrap();
                    let index_map = self.index.h3.get(&res_u8).unwrap();
                    if let Ok(center_coord) = LatLng::new(lat, lon) {
                        let center_cell = center_coord.to_cell(res);
                        let mut geo_candidates = RoaringBitmap::new();
                        let target_candidates = if group_by.is_some() {
                            (limit_val * 20).max(1000) as u64
                        } else {
                            limit_val as u64
                        };

                        // If radius_m is present, estimate the maximum number of rings k
                        // Distance between H3 cell centers at resolution 10 is about 1.1km
                        // To be safe, we can use a large limit for k, or stop as soon as we exceed radius_m.

                        debug_log!(
                            "[{:.3?}] Adaptive H3 starting (target: {}, radius_m: {:?})",
                            start_time.elapsed(),
                            target_candidates,
                            radius_m
                        );
                        for k in 0..200 {
                            let ring: Vec<h3o::CellIndex> = center_cell.grid_disk(k);
                            for cell in ring {
                                if let Some(bitmap) = index_map.get(&u64::from(cell)) {
                                    geo_candidates |= bitmap;
                                }
                            }

                            // If we have a max radius, we could stop early if ring k
                            // significantly exceeds the radius. However, intersection with `candidates`
                            // will later filter by radius if we use spatial_refinement.
                            // For now, we focus on having enough candidates.

                            let intersection = geo_candidates.clone() & candidates.clone();
                            if intersection.len() >= target_candidates && k >= 1 {
                                debug_log!("[{:.3?}] Adaptive H3 found {} candidates at k={} (took {:.3?})", start_time.elapsed(), intersection.len(), k, adaptive_start.elapsed());
                                candidates = intersection;
                                break;
                            }
                            if k == 199 {
                                debug_log!("[{:.3?}] Adaptive H3 reached max radius (k=199), found {} candidates", start_time.elapsed(), intersection.len());
                                candidates = intersection;
                            }
                        }
                    }
                }
            }
        }

        let mut fgb_distances = HashMap::new();
        let using_fgb = !self.index.fgb.is_empty() && (center.is_some() || bbox.is_some());

        let mut spatial_done = false;
        // Spatial search via FlatGeoBuf (if available)
        if using_fgb {
            // We look for the first available FGB index
            if let Some((field, fgb_mmap)) = self.index.fgb.iter().next() {
                debug_log!(
                    "[{:.3?}] FGB search starting on field '{}'",
                    start_time.elapsed(),
                    field
                );
                let mut cursor = std::io::Cursor::new(fgb_mmap.as_ref());
                let fgb_reader = flatgeobuf::FgbReader::open(&mut cursor)?;

                let search_bbox = if let Some(b) = bbox {
                    b
                } else if let (Some([lat, lon]), Some(r_m)) = (center, radius_m) {
                    // Quick BBox approximation for the circle
                    let lat_deg = r_m / 111320.0;
                    let lon_deg = r_m / (111320.0 * (lat.to_radians().cos()));
                    [lon - lon_deg, lat - lat_deg, lon + lon_deg, lat + lat_deg]
                } else if let Some([lat, lon]) = center {
                    [lon - 0.0001, lat - 0.0001, lon + 0.0001, lat + 0.0001]
                } else {
                    [0.0, 0.0, 0.0, 0.0]
                };

                let mut fgb_candidates = RoaringBitmap::new();
                let mut features = fgb_reader.select_bbox(
                    search_bbox[0],
                    search_bbox[1],
                    search_bbox[2],
                    search_bbox[3],
                )?;

                let center_point = center.map(|[lat, lon]| geo::Point::new(lon, lat));

                while let Some(feature) = features.next()? {
                    let feat_id: u64 = feature.property_n(0)?;
                    if candidates.is_empty() || candidates.contains(feat_id as u32) {
                        if let Some(cp) = center_point {
                            let geo_obj = feature.to_geo()?;
                            let distance = match &geo_obj {
                                geo::Geometry::Polygon(poly) => {
                                    if poly.contains(&cp) {
                                        0.0
                                    } else {
                                        match poly.closest_point(&cp) {
                                            geo::Closest::SinglePoint(p)
                                            | geo::Closest::Intersection(p) => {
                                                Haversine.distance(p, cp)
                                            }
                                            _ => f64::INFINITY,
                                        }
                                    }
                                }
                                geo::Geometry::MultiPolygon(mpoly) => {
                                    if mpoly.contains(&cp) {
                                        0.0
                                    } else {
                                        match mpoly.closest_point(&cp) {
                                            geo::Closest::SinglePoint(p)
                                            | geo::Closest::Intersection(p) => {
                                                Haversine.distance(p, cp)
                                            }
                                            _ => f64::INFINITY,
                                        }
                                    }
                                }
                                _ => f64::INFINITY,
                            };

                            fgb_distances.insert(feat_id, distance);

                            let matches = if let Some(r_m) = radius_m {
                                distance <= r_m
                            } else {
                                // If no radius, consider it a match for now
                                // Distance sorting will happen later
                                true
                            };

                            if matches {
                                fgb_candidates.insert(feat_id as u32);
                            }
                        } else if bbox.is_some() {
                            // If only bbox, keep everything
                            fgb_candidates.insert(feat_id as u32);
                        }
                    }
                }

                if candidates.is_empty() {
                    candidates = fgb_candidates;
                } else {
                    candidates &= fgb_candidates;
                }
                debug_log!(
                    "[{:.3?}] FGB search done, candidates: {}",
                    start_time.elapsed(),
                    candidates.len()
                );
                spatial_done = true;
            }
        }

        if !spatial_done && (center.is_some() || radius_m.is_some()) {
            if let (Some(center_pt), Some(radius_val)) = (center, radius_m) {
                let r = 6371000.0;
                let center_ecef = {
                    let lat_rad = center_pt[0].to_radians();
                    let lon_rad = center_pt[1].to_radians();
                    let x = r * lat_rad.cos() * lon_rad.cos();
                    let y = r * lat_rad.cos() * lon_rad.sin();
                    let z = r * lat_rad.sin();
                    Some((x as f32, y as f32, z as f32))
                };
                let radius_sq = Some(radius_val * radius_val);

                let mut filtered = RoaringBitmap::new();
                for id in candidates.iter() {
                    if let Some(ecef) = self.index.get_ecef(id as u64) {
                        let [x, y, z] = ecef;
                        if let (Some(c), Some(r2)) = (center_ecef, radius_sq) {
                            let dx = c.0 - x;
                            let dy = c.1 - y;
                            let dz = c.2 - z;
                            if (dx * dx + dy * dy + dz * dz) as f64 <= r2 {
                                filtered.insert(id);
                            }
                        }
                    }
                }
                candidates = filtered;
                debug_log!(
                    "[{:.3?}] Spatial refinement done, candidates: {}",
                    start_time.elapsed(),
                    candidates.len()
                );
            }
        }

        if let Some(gb_field) = group_by {
            return self.search_group_by(
                candidates,
                gb_field,
                limit.unwrap_or(10),
                center,
                start_time,
            );
        }

        let mut res = Vec::new();
        let take_limit = limit.unwrap_or(100);

        if let Some(center_pt) = center {
            if using_fgb {
                let mut sorted_by_dist: Vec<(u64, f64)> = candidates
                    .iter()
                    .map(|id| {
                        let dist = fgb_distances
                            .get(&(id as u64))
                            .copied()
                            .unwrap_or(f64::INFINITY);
                        (id as u64, dist)
                    })
                    .collect();
                sorted_by_dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                res = sorted_by_dist.into_iter().take(take_limit).collect();
            } else {
                let mut heap = BinaryHeap::with_capacity(take_limit);
                let r = 6371000.0;
                let lat_rad = center_pt[0].to_radians();
                let lon_rad = center_pt[1].to_radians();
                let cx = (r * lat_rad.cos() * lon_rad.cos()) as f32;
                let cy = (r * lat_rad.cos() * lon_rad.sin()) as f32;
                let cz = (r * lat_rad.sin()) as f32;

                for id in candidates.iter() {
                    let dist = if let Some(p) = self.index.get_ecef(id as u64) {
                        let dx = cx - p[0];
                        let dy = cy - p[1];
                        let dz = cz - p[2];
                        (dx * dx + dy * dy + dz * dz).sqrt() as f64
                    } else {
                        0.0
                    };

                    if heap.len() < take_limit {
                        heap.push((OrderedFloat(dist), id as u64));
                    } else if let Some(mut top) = heap.peek_mut() {
                        if dist < top.0 .0 {
                            top.0 = OrderedFloat(dist);
                            top.1 = id as u64;
                        }
                    }
                }
                let sorted_res: Vec<_> = heap.into_sorted_vec();
                res = sorted_res.into_iter().map(|(d, id)| (id, d.0)).collect();
            }
        } else {
            for id in candidates.iter().take(take_limit) {
                res.push((id as u64, 0.0));
            }
        }

        debug_log!("[{:.3?}] Search completed", start_time.elapsed());
        Ok(res)
    }

    fn search_group_by(
        &self,
        candidates: RoaringBitmap,
        gb_field: &str,
        limit_val: usize,
        center: Option<[f64; 2]>,
        start_time: Instant,
    ) -> Result<Vec<(u64, f64)>> {
        debug_log!(
            "[{:.3?}] GroupBy starting on {} candidates for field '{}' (bitmap-based)",
            start_time.elapsed(),
            candidates.len(),
            gb_field
        );

        let group_index = if let Some(idx) = self.index.secondary.get(gb_field) {
            idx
        } else {
            return Ok(Vec::new());
        };

        let center_ecef = center.map(|[lat, lon]| {
            let r = 6371000.0;
            let lat_rad = lat.to_radians();
            let lon_rad = lon.to_radians();
            let x = (r * lat_rad.cos() * lon_rad.cos()) as f32;
            let y = (r * lat_rad.cos() * lon_rad.sin()) as f32;
            let z = (r * lat_rad.sin()) as f32;
            (x, y, z)
        });

        let mut res = Vec::new();

        for (_group_value, group_bitmap) in group_index {
            let match_bitmap = &candidates & group_bitmap;
            if !match_bitmap.is_empty() {
                // Collect the best candidates from this group
                if let Some(c) = center_ecef {
                    let mut group_results = Vec::new();
                    for id in match_bitmap.iter() {
                        let dist_sq = if let Some(p) = self.index.get_ecef(id as u64) {
                            let dx = c.0 - p[0];
                            let dy = c.1 - p[1];
                            let dz = c.2 - p[2];
                            dx * dx + dy * dy + dz * dz
                        } else {
                            f32::INFINITY
                        };
                        group_results.push((id as u64, dist_sq));
                    }
                    // Sort by distance for this group and take top_n
                    group_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    for (id, dist_sq) in group_results.into_iter().take(limit_val) {
                        res.push((id, (dist_sq as f64).sqrt()));
                    }
                } else {
                    // No center, just take the top_n first IDs from the bitmap
                    for id in match_bitmap.iter().take(limit_val) {
                        res.push((id as u64, 0.0));
                    }
                }
            }
        }

        // Final global sorting (optional depending on need, but here we keep by distance if center present)
        if center_ecef.is_some() {
            res.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        }
        let final_res = res;

        debug_log!(
            "[{:.3?}] Search completed (GroupBy), found {} groups, {} total results",
            start_time.elapsed(),
            final_res.len(),
            final_res.len()
        );
        Ok(final_res)
    }
}

/// A retrieved document with its raw serialized data and ID.
pub struct Item<'a> {
    /// Raw FlatBuffers‑serialized entry data
    pub data: &'a [u8],
    /// Document ID (1‑based)
    pub id: u64,
}

impl<'a> Item<'a> {
    pub fn to_json_value(&self) -> JsonValue {
        let entry = schema_generated::EntryRef::read_as_root(self.data).unwrap();
        let mut map = serde_json::Map::new();
        map.insert("id".to_string(), json!(self.id));
        if let Ok(Some(fields)) = entry.fields() {
            for field_res in fields {
                let field = field_res.unwrap();
                let name = field.key().unwrap().unwrap().to_string();
                let value_ref = field.value().unwrap().unwrap();
                let value = match value_ref {
                    schema_generated::ValueRef::Text(t) => json!(t.value().unwrap().unwrap()),
                    schema_generated::ValueRef::Integer(i) => json!(i.value().unwrap()),
                    schema_generated::ValueRef::Float(f) => json!(f.value().unwrap()),
                    schema_generated::ValueRef::Boolean(b) => json!(b.value().unwrap()),
                    schema_generated::ValueRef::Date(d) => {
                        let days = d.value().unwrap();
                        let date = chrono::NaiveDate::from_num_days_from_ce_opt(days + 719163)
                            .unwrap_or_default();
                        json!(date.format("%Y-%m-%d").to_string())
                    }
                    schema_generated::ValueRef::DateTime(dt) => {
                        let seconds = dt.value().unwrap();
                        if let Some(datetime) = chrono::DateTime::from_timestamp(seconds, 0) {
                            json!(datetime.to_rfc3339())
                        } else {
                            json!(seconds)
                        }
                    }
                };
                map.insert(name, value);
            }
        }
        if let Ok(Some(wkb_data)) = entry.geometry() {
            let mut json_bytes = Vec::new();
            let mut writer = geozero::geojson::GeoJsonWriter::new(&mut json_bytes);
            use geozero::GeozeroGeometry;
            if let Ok(_) = Wkb(wkb_data.to_vec()).process_geom(&mut writer) {
                if let Ok(val) = serde_json::from_slice(&json_bytes) {
                    map.insert("geometry".to_string(), val);
                }
            }
        }
        JsonValue::Object(map)
    }

    pub fn to_json(&self) -> String {
        self.to_json_value().to_string()
    }

    pub fn to_geojson_value(&self, engine: &Engine) -> JsonValue {
        let mut props = serde_json::Map::new();
        props.insert("id".to_string(), json!(self.id));
        let entry = schema_generated::EntryRef::read_as_root(self.data).unwrap();
        if let Ok(Some(fields)) = entry.fields() {
            for field_res in fields {
                let field = field_res.unwrap();
                let name = field.key().unwrap().unwrap().to_string();
                let value_ref = field.value().unwrap().unwrap();
                let value = match value_ref {
                    schema_generated::ValueRef::Text(t) => json!(t.value().unwrap().unwrap()),
                    schema_generated::ValueRef::Integer(i) => json!(i.value().unwrap()),
                    schema_generated::ValueRef::Float(f) => json!(f.value().unwrap()),
                    schema_generated::ValueRef::Boolean(b) => json!(b.value().unwrap()),
                    schema_generated::ValueRef::Date(d) => {
                        let days = d.value().unwrap();
                        let date = chrono::NaiveDate::from_num_days_from_ce_opt(days + 719163)
                            .unwrap_or_default();
                        json!(date.format("%Y-%m-%d").to_string())
                    }
                    schema_generated::ValueRef::DateTime(dt) => {
                        let seconds = dt.value().unwrap();
                        if let Some(datetime) = chrono::DateTime::from_timestamp(seconds, 0) {
                            json!(datetime.to_rfc3339())
                        } else {
                            json!(seconds)
                        }
                    }
                };
                props.insert(name, value);
            }
        }

        let mut geometry = JsonValue::Null;
        if let Ok(Some(wkb_data)) = entry.geometry() {
            let mut json_bytes = Vec::new();
            let mut writer = geozero::geojson::GeoJsonWriter::new(&mut json_bytes);
            use geozero::GeozeroGeometry;
            if let Ok(_) = Wkb(wkb_data.to_vec()).process_geom(&mut writer) {
                if let Ok(val) = serde_json::from_slice(&json_bytes) {
                    geometry = val;
                }
            }
        }

        if geometry.is_null() {
            if let Some(p) = engine.get_ecef(self.id) {
                let r = 6371000.0;
                let lat = (p[2] as f64 / r).asin().to_degrees();
                let lon = (p[1] as f64).atan2(p[0] as f64).to_degrees();
                geometry = json!({
                    "type": "Point",
                    "coordinates": [lon, lat]
                });
            }
        }

        json!({
            "type": "Feature",
            "geometry": geometry,
            "properties": props
        })
    }
}

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
use planus::ReadAsRoot;
use roaring::RoaringBitmap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::path::Path;
use std::time::Instant;

/// Convert text to a normalized slug: remove accents, lowercase, keep only ASCII alphanumeric.
pub fn slugify(text: &str) -> String {
    let mut s = deunicode(text).to_lowercase();
    s.retain(|c| c.is_ascii_alphanumeric() || c == ' ');
    s
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    pub stopwords: HashMap<String, HashSet<String>>,
}

pub struct MetadataStore {
    offsets_mmap: Mmap,
    data_mmap: Mmap,
}

impl MetadataStore {
    pub fn new<P: AsRef<Path>>(directory: P) -> Result<Self> {
        let dir = directory.as_ref();
        let offsets_file = File::open(dir.join("data.offsets"))?;
        let offsets_mmap = unsafe { Mmap::map(&offsets_file)? };
        let data_file = File::open(dir.join("data.bin"))?;
        let data_mmap = unsafe { Mmap::map(&data_file)? };
        Ok(Self {
            offsets_mmap,
            data_mmap,
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

pub struct RangeIndex {
    pub boundaries: Vec<f64>,
    pub bitmaps: Vec<RoaringBitmap>,
    pub values_mmap: Option<Mmap>,
}

#[derive(Debug, Clone)]
pub struct RangeQuery {
    pub min: Option<String>,
    pub max: Option<String>,
}

pub struct ColumnarIndex {
    pub mmap: Mmap,
    pub dict: Vec<String>,
    pub width: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexMetadata {
    pub name: String,
    pub r#type: String,
    pub field: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SourceMetadata {
    pub name: String,
    pub size: u64,
    pub indexes: Vec<IndexMetadata>,
}

pub struct IndexStore {
    pub metadata: Option<SourceMetadata>,
    pub runtime_config: RuntimeConfig,
    pub h3: HashMap<u8, HashMap<u64, RoaringBitmap>>,
    pub secondary: HashMap<String, HashMap<String, RoaringBitmap>>,
    pub columnar: HashMap<String, ColumnarIndex>,
    pub fst_mmaps: HashMap<String, (Mmap, Mmap, Option<Mmap>)>,
    pub coords_mmap: Option<Mmap>,
    pub ranges: HashMap<String, RangeIndex>,
    pub primary_mmap: Option<Mmap>,
}

impl IndexStore {
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

        let mut columnar = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.ends_with(".col") {
                        let field = filename.trim_end_matches(".col").to_string();
                        let dict_path = dir.join(format!("{}.dict", field));
                        if let (Ok(col_file), Ok(mut dict_file)) =
                            (File::open(&path), File::open(dict_path))
                        {
                            let mmap = unsafe { Mmap::map(&col_file)? };
                            let width = mmap[0];
                            use std::io::Read;
                            let mut dict = Vec::new();
                            let mut dict_buf = Vec::new();
                            dict_file.read_to_end(&mut dict_buf)?;
                            let mut cursor = std::io::Cursor::new(dict_buf);
                            while cursor.position() < cursor.get_ref().len() as u64 {
                                let mut len_buf = [0u8; 4];
                                cursor.read_exact(&mut len_buf)?;
                                let len = u32::from_le_bytes(len_buf) as usize;
                                let mut val_buf = vec![0u8; len];
                                cursor.read_exact(&mut val_buf)?;
                                dict.push(String::from_utf8(val_buf)?);
                            }
                            columnar.insert(field, ColumnarIndex { mmap, dict, width });
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
            columnar,
            fst_mmaps,
            coords_mmap,
            ranges,
            primary_mmap,
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

    pub fn get_column_value(&self, id: u64, field: &str) -> Option<&str> {
        let col = self.columnar.get(field)?;
        let idx = (id - 1) as usize;
        let start = 1 + idx * col.width as usize;
        if start + col.width as usize > col.mmap.len() {
            return None;
        }
        let dict_idx = match col.width {
            1 => col.mmap[start] as usize,
            2 => u16::from_le_bytes(col.mmap[start..start + 2].try_into().unwrap()) as usize,
            4 => u32::from_le_bytes(col.mmap[start..start + 4].try_into().unwrap()) as usize,
            _ => return None,
        };
        col.dict.get(dict_idx).map(|s| s.as_str())
    }
}

pub struct Engine {
    pub name: String,
    pub metadata: MetadataStore,
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
        let mut all = RoaringBitmap::new();
        if let Some(coords) = &self.index.coords_mmap {
            let num_docs = (coords.len() / 12) as u32;
            for i in 1..=num_docs {
                all.insert(i);
            }
        }
        all
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

        if let (Some([lat, lon]), None, Some(limit_val)) = (center, radius_m, limit) {
            if bbox.is_none() {
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
                        debug_log!(
                            "[{:.3?}] Adaptive H3 starting (target: {})",
                            start_time.elapsed(),
                            target_candidates
                        );
                        for k in 0..100 {
                            let ring: Vec<h3o::CellIndex> = center_cell.grid_disk(k);
                            for cell in ring {
                                if let Some(bitmap) = index_map.get(&u64::from(cell)) {
                                    geo_candidates |= bitmap;
                                }
                            }
                            let intersection = geo_candidates.clone() & candidates.clone();
                            if intersection.len() >= target_candidates && k >= 1 {
                                debug_log!("[{:.3?}] Adaptive H3 found {} candidates at k={} (took {:.3?})", start_time.elapsed(), intersection.len(), k, adaptive_start.elapsed());
                                candidates = intersection;
                                break;
                            }
                            if k == 99 {
                                debug_log!("[{:.3?}] Adaptive H3 reached max radius (k=99), found {} candidates", start_time.elapsed(), intersection.len());
                                candidates = intersection;
                            }
                        }
                    }
                }
            }
        }

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
        for id in candidates.iter().take(limit.unwrap_or(100)) {
            let dist = if let Some(center_pt) = center {
                if let Some(p) = self.index.get_ecef(id as u64) {
                    let r = 6371000.0;
                    let lat_rad = center_pt[0].to_radians();
                    let lon_rad = center_pt[1].to_radians();
                    let cx = (r * lat_rad.cos() * lon_rad.cos()) as f32;
                    let cy = (r * lat_rad.cos() * lon_rad.sin()) as f32;
                    let cz = (r * lat_rad.sin()) as f32;
                    let dx = cx - p[0];
                    let dy = cy - p[1];
                    let dz = cz - p[2];
                    (dx * dx + dy * dy + dz * dz).sqrt() as f64
                } else {
                    0.0
                }
            } else {
                0.0
            };
            res.push((id as u64, dist));
        }

        debug_log!("[{:.3?}] Search completed", start_time.elapsed());
        Ok(res)
    }

    fn search_group_by(
        &self,
        results: RoaringBitmap,
        gb_field: &str,
        limit_val: usize,
        center: Option<[f64; 2]>,
        start_time: Instant,
    ) -> Result<Vec<(u64, f64)>> {
        debug_log!(
            "[{:.3?}] GroupBy starting on {} candidates for field '{}'",
            start_time.elapsed(),
            results.len(),
            gb_field
        );
        let center_ecef = center.map(|[lat, lon]| {
            let r = 6371000.0;
            let lat_rad = lat.to_radians();
            let lon_rad = lon.to_radians();
            let x = (r * lat_rad.cos() * lon_rad.cos()) as f32;
            let y = (r * lat_rad.cos() * lon_rad.sin()) as f32;
            let z = (r * lat_rad.sin()) as f32;
            (x, y, z)
        });

        use ordered_float::OrderedFloat;
        use std::collections::BinaryHeap;

        let mut groups: HashMap<String, BinaryHeap<(OrderedFloat<f32>, u64)>> = HashMap::new();
        for id in results.iter() {
            if let Some(val) = self.index.get_column_value(id as u64, gb_field) {
                let dist_sq =
                    if let (Some(c), Some(p)) = (center_ecef, self.index.get_ecef(id as u64)) {
                        let dx = c.0 - p[0];
                        let dy = c.1 - p[1];
                        let dz = c.2 - p[2];
                        dx * dx + dy * dy + dz * dz
                    } else {
                        0.0
                    };

                let heap = groups.entry(val.to_string()).or_default();
                heap.push((OrderedFloat(-dist_sq), id as u64));
            }
        }
        let mut res = Vec::new();
        let groups_count = groups.len();
        for (_, mut heap) in groups {
            if let Some((neg_dist_sq, id)) = heap.pop() {
                res.push((id, (-neg_dist_sq.0).sqrt() as f64));
            }
        }
        res.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let final_res: Vec<_> = res.into_iter().take(limit_val).collect();

        debug_log!(
            "[{:.3?}] Search completed (GroupBy), found {} groups, {} total results",
            start_time.elapsed(),
            groups_count,
            final_res.len()
        );
        Ok(final_res)
    }
}

pub struct Item<'a> {
    pub data: &'a [u8],
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

        let geometry = if let Some(p) = engine.get_ecef(self.id) {
            let r = 6371000.0;
            let lat = (p[2] as f64 / r).asin().to_degrees();
            let lon = (p[1] as f64).atan2(p[0] as f64).to_degrees();
            json!({
                "type": "Point",
                "coordinates": [lon, lat]
            })
        } else {
            JsonValue::Null
        };

        json!({
            "type": "Feature",
            "geometry": geometry,
            "properties": props
        })
    }
}

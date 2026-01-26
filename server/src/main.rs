use anyhow::Result;
use arc_swap::ArcSwap;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use clap::Parser;
use cold_search_core::Engine;
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:3001")]
    http_addr: String,

    #[arg(long, help = "Enable debug logs")]
    debug: bool,

    #[arg(help = "Directories containing search indices")]
    index_dirs: Vec<PathBuf>,
}

// State for multi-collection support
pub struct AppState {
    pub collections: ArcSwap<HashMap<String, Engine>>,
}

// HTTP Implementation
#[derive(Serialize)]
struct IndexInfo {
    name: String,
    r#type: String,
    field: String,
}

#[derive(Serialize)]
struct CollectionInfo {
    name: String,
    size: u64,
    indexes: Vec<IndexInfo>,
}

async fn http_get(
    State(state): State<Arc<AppState>>,
    AxumPath((collection_name, key)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let collections = state.collections.load();
    let engine = match collections.get(&collection_name) {
        Some(e) => e,
        None => {
            return (
                StatusCode::NOT_FOUND,
                format!("Collection {} not found", collection_name),
            )
                .into_response()
        }
    };

    // Try primary key first
    if let Some(item) = engine.get_by_key(&key) {
        return Json(item.to_json_value()).into_response();
    }

    // Fallback to internal ID
    if let Ok(id) = key.parse::<u64>() {
        if let Some(item) = engine.get(id) {
            return Json(item.to_json_value()).into_response();
        }
    }

    (StatusCode::NOT_FOUND, format!("Item {} not found", key)).into_response()
}

async fn http_search(
    State(state): State<Arc<AppState>>,
    AxumPath(collection_name): AxumPath<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let collections = state.collections.load();
    let engine = match collections.get(&collection_name) {
        Some(e) => e,
        None => {
            return (
                StatusCode::NOT_FOUND,
                format!("Collection {} not found", collection_name),
            )
                .into_response()
        }
    };

    let text = params.get("q").cloned();

    // Spatial parameters
    let lat = params.get("lat").and_then(|v| v.parse::<f64>().ok());
    let lon = params.get("lon").and_then(|v| v.parse::<f64>().ok());
    let radius = params.get("radius").and_then(|v| v.parse::<f64>().ok());

    let min_lat = params.get("min_lat").and_then(|v| v.parse::<f64>().ok());
    let min_lon = params.get("min_lon").and_then(|v| v.parse::<f64>().ok());
    let max_lat = params.get("max_lat").and_then(|v| v.parse::<f64>().ok());
    let max_lon = params.get("max_lon").and_then(|v| v.parse::<f64>().ok());

    let center = if let (Some(la), Some(lo)) = (lat, lon) {
        Some([la, lo])
    } else {
        None
    };

    let bbox = if let (Some(min_la), Some(min_lo), Some(max_la), Some(max_lo)) =
        (min_lat, min_lon, max_lat, max_lon)
    {
        Some([min_lo, min_la, max_lo, max_la])
    } else {
        None
    };

    let filter_by = params
        .get("filter_by")
        .or_else(|| params.get("filter"))
        .map(|s| s.as_str())
        .unwrap_or("");
    let filters = cold_search_core::filter::parse_filter_by(filter_by);

    let top_n = params.get("top_n").and_then(|v| v.parse::<usize>().ok());
    let group_by = params.get("group_by").cloned();
    let output = params
        .get("output")
        .map(|s| s.as_str())
        .unwrap_or("geojson");

    match engine.search(
        filters,
        bbox,
        text.as_deref(),
        group_by.as_deref(),
        top_n,
        center,
        radius,
    ) {
        Ok(results) => {
            if output == "json" {
                let mut output_results = Vec::new();
                for (id, dist) in results {
                    if let Some(item) = engine.get(id) {
                        let mut val = item.to_json_value();
                        if let Some(obj) = val.as_object_mut() {
                            obj.insert("__distance".to_string(), serde_json::json!(dist));

                            if let Some(p) = engine.index.get_ecef(id) {
                                let r = 6371000.0;
                                let lat = (p[2] as f64 / r).asin().to_degrees();
                                let lon = (p[1] as f64).atan2(p[0] as f64).to_degrees();
                                obj.insert("lat".to_string(), serde_json::json!(lat));
                                obj.insert("lon".to_string(), serde_json::json!(lon));
                            }
                        }
                        output_results.push(val);
                    }
                }
                Json(output_results).into_response()
            } else {
                // GeoJSON (default)
                let features: Vec<_> = results
                    .into_iter()
                    .filter_map(|(id, dist)| {
                        engine.get(id).map(|item| {
                            let mut val = item.to_geojson_value(engine);
                            if let Some(obj) = val.as_object_mut() {
                                if let Some(props) =
                                    obj.get_mut("properties").and_then(|p| p.as_object_mut())
                                {
                                    props.insert("__distance".to_string(), serde_json::json!(dist));
                                }
                            }
                            val
                        })
                    })
                    .collect();

                Json(serde_json::json!({
                    "type": "FeatureCollection",
                    "features": features
                }))
                .into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn http_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let collections = state.collections.load();
    let infos: Vec<CollectionInfo> =
        collections
            .values()
            .map(|engine| {
                let metadata = engine.index.metadata.clone().unwrap_or_else(|| {
                    cold_search_core::SourceMetadata {
                        name: engine.name.clone(),
                        size: 0,
                        indexes: Vec::new(),
                        geometry_type: None,
                    }
                });

                CollectionInfo {
                    name: metadata.name,
                    size: metadata.size,
                    indexes: metadata
                        .indexes
                        .into_iter()
                        .map(|i| IndexInfo {
                            name: i.name,
                            r#type: i.r#type,
                            field: i.field,
                        })
                        .collect(),
                }
            })
            .collect();
    Json(infos)
}

fn load_collections(paths: &[PathBuf]) -> HashMap<String, Engine> {
    let mut collections = HashMap::new();
    for path in paths {
        if !path.exists() {
            continue;
        }

        // 1. Check if the path itself is a collection
        if path.join("data.bin").exists() && path.join("data.offsets").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                println!("Loading collection: {} from {:?}", name, path);
                match Engine::new(name.to_string(), path) {
                    Ok(engine) => {
                        println!("  Indices: {:?}", engine.info());
                        collections.insert(name.to_string(), engine);
                    }
                    Err(e) => eprintln!("  Failed to load {}: {}", name, e),
                }
            }
        } else if path.is_dir() {
            // 2. Otherwise, check subdirectories for collections
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let sub_path = entry.path();
                    if sub_path.is_dir()
                        && sub_path.join("data.bin").exists()
                        && sub_path.join("data.offsets").exists()
                    {
                        if let Some(name) = sub_path.file_name().and_then(|n| n.to_str()) {
                            println!(
                                "Loading collection from root: {} from {:?}",
                                name, &sub_path
                            );
                            match Engine::new(name.to_string(), &sub_path) {
                                Ok(engine) => {
                                    println!("  Indices: {:?}", engine.info());
                                    collections.insert(name.to_string(), engine);
                                }
                                Err(e) => eprintln!("  Failed to load {}: {}", name, e),
                            }
                        }
                    }
                }
            }
        }
    }
    collections
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    cold_search_core::set_debug(args.debug);
    let http_addr: SocketAddr = args.http_addr.parse()?;

    let index_paths = if args.index_dirs.is_empty() {
        vec![PathBuf::from("data_out"), PathBuf::from("data_bano")]
    } else {
        args.index_dirs
    };

    let collections = load_collections(&index_paths);
    let state = Arc::new(AppState {
        collections: ArcSwap::from(Arc::new(collections)),
    });

    // Setup Watcher
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let state_to_swap = state.clone();
    let watch_paths = index_paths.clone();

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                // Trigger reload for any relevant change (Creation, Modification, Removal, Renaming)
                // We ignore Access events to avoid unnecessary reloads
                if !event.kind.is_access() {
                    let _ = tx.blocking_send(());
                }
            }
        })?;

    use notify::Watcher;
    for path in &index_paths {
        if path.exists() {
            println!("Watching for changes in: {:?}", path);
            // Recursive mode to catch changes inside collection directories
            watcher.watch(path, notify::RecursiveMode::Recursive)?;
        }
    }

    // Hot-reload loop
    tokio::spawn(async move {
        while let Some(_) = rx.recv().await {
            println!("Reloading collections...");
            let new_collections = load_collections(&watch_paths);
            state_to_swap.collections.store(Arc::new(new_collections));
            println!("Collections reloaded successfully.");
        }
    });

    // Start Axum HTTP server
    let app = Router::new()
        .route("/health", get(http_health))
        .route("/:source/search", get(http_search))
        .route("/:source/get/:id", get(http_get))
        .with_state(state.clone());

    println!("HTTP Server listening on http://{}", http_addr);
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

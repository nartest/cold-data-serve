use anyhow::Result;
use clap::Parser;
use cold_search_core::config_v2::ConfigV2;
use std::path::PathBuf;
use sysinfo::System;

mod pipeline;
mod query_builder;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the YAML profile (v2 format)
    #[arg(short, long)]
    profile: PathBuf,

    /// Output directory for index files
    #[arg(short, long)]
    output: PathBuf,

    /// Optional input file override (if not specified in profile)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Enable debug logs
    #[arg(long)]
    debug: bool,

    /// Memory limit for DuckDB (e.g., "128GB", "80%"). Defaults to 70% of total RAM.
    #[arg(short, long)]
    memory: Option<String>,

    /// Number of threads for DuckDB. Defaults to number of physical CPUs - 2.
    #[arg(short, long)]
    threads: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    cold_search_core::set_debug(args.debug);

    println!("Loading profile from: {:?}", args.profile);
    let mut config = ConfigV2::from_yaml(&args.profile)?;

    if let Some(input_override) = args.input {
        config.source.path = input_override;
    }

    config.validate()?;

    println!("Indexing collection: {}", config.collection);
    println!("Output directory: {:?}", args.output);

    // Détection dynamique des ressources
    let mut sys = System::new_all();
    sys.refresh_memory();

    let final_threads = args.threads.unwrap_or_else(|| {
        let cpus = sys.physical_core_count().unwrap_or(4);
        if cpus > 2 {
            cpus - 2
        } else {
            1
        }
    });

    let final_memory = args.memory.unwrap_or_else(|| {
        let total_ram = sys.total_memory();
        let target_ram = (total_ram as f64 * 0.7) as u64;
        let gb = target_ram / (1024 * 1024 * 1024);
        format!("{}GB", gb)
    });

    println!("Resource allocation:");
    println!("  - Threads: {}", final_threads);
    println!("  - Memory limit: {}", final_memory);

    let pipeline = pipeline::Pipeline::new(config, args.output, final_memory, final_threads)?;
    pipeline.run()?;

    println!("Indexing completed successfully!");

    Ok(())
}

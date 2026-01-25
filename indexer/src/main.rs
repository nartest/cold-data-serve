use anyhow::Result;
use clap::Parser;
use cold_search_core::config_v2::ConfigV2;
use std::path::PathBuf;

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

    let pipeline = pipeline::Pipeline::new(config, args.output)?;
    pipeline.run()?;

    println!("Indexing completed successfully!");

    Ok(())
}

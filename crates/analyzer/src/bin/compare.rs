use analyzer::{
    csv_reader::{create_fallback_data, read_key_freq_from_directory},
    geometry::{Geometry, GeometryName},
    keys::ParseOptions,
    optimize::{Config, common::execute_comparison},
};
use clap::Parser;
use log::{error, info, warn};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Compare v1 and v2 keyboard optimization results", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short = 'c', long = "config", default_value = "config/default.toml")]
    config: PathBuf,

    /// CSV data directory path (fallback to single file if directory doesn't exist)
    #[arg(short = 'd', long = "data", default_value = "csv")]
    data_path: PathBuf,

    /// Keyboard geometry type
    #[arg(short = 'g', long = "geometry", default_value = "row-stagger")]
    geometry: String,

    /// Report format for comparison results
    #[arg(short = 'r', long = "report", default_value = "html")]
    report_format: String,

    /// Include F-keys in optimization
    #[arg(short = 'f', long = "include-fkeys")]
    include_fkeys: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    // Parse geometry type
    let geometry_enum = match args.geometry.as_str() {
        "row-stagger" => GeometryName::RowStagger,
        "ortho" => GeometryName::Ortho,
        "column-stagger" => {
            error!("Geometry type 'column-stagger' is not yet supported.");
            std::process::exit(1);
        }
        _ => {
            error!(
                "Unknown geometry type: {}. Available: row-stagger, ortho, column-stagger",
                args.geometry
            );
            std::process::exit(1);
        }
    };

    // Build geometry
    let geom = Geometry::build(geometry_enum)?;

    // Load configuration
    let mut config = if args.config.exists() {
        Config::load_from_file(args.config.to_str().unwrap())?
    } else {
        warn!(
            "Config file not found: {}, using default settings",
            args.config.display()
        );
        Config::default()
    };

    // Override config for comparison mode
    config.solver.version = "compare".to_string();
    config.comparison = Some(analyzer::optimize::config::ComparisonConfig {
        enable_parallel: false,
        save_report: true,
        report_format: args.report_format.clone(),
    });

    // Override F-keys setting if provided
    if args.include_fkeys {
        config.v1.include_fkeys = true;
    }

    // Load key frequency data
    let parse_opts = ParseOptions {
        include_fkeys: config.v1.include_fkeys,
        ..Default::default()
    };

    let freqs = if args.data_path.is_dir() {
        // Try to read from directory first
        match read_key_freq_from_directory(&args.data_path, &parse_opts) {
            Ok(freq) => {
                info!(
                    "Successfully loaded {} unique keys from {} CSV files",
                    freq.unique_keys(),
                    args.data_path.display()
                );
                info!("Total key presses: {}", freq.total());
                freq
            }
            Err(e) => {
                warn!(
                    "Failed to read CSV files from directory {}: {}",
                    args.data_path.display(),
                    e
                );
                warn!("Using fallback test data instead.");
                create_fallback_data()
            }
        }
    } else if args.data_path.is_file() {
        // Single CSV file
        match analyzer::csv_reader::read_key_freq_csv(&args.data_path, &parse_opts) {
            Ok(freq) => {
                info!(
                    "Successfully loaded {} unique keys from CSV file: {}",
                    freq.unique_keys(),
                    args.data_path.display()
                );
                info!("Total key presses: {}", freq.total());
                freq
            }
            Err(e) => {
                warn!(
                    "Failed to read CSV file {}: {}",
                    args.data_path.display(),
                    e
                );
                warn!("Using fallback test data instead.");
                create_fallback_data()
            }
        }
    } else {
        warn!(
            "Data path {} not found. Using fallback test data.",
            args.data_path.display()
        );
        create_fallback_data()
    };

    info!("=== v1 vs v2 Keyboard Layout Optimization Comparison ===");
    info!("Configuration: {}", args.config.display());
    info!("Data source: {}", args.data_path.display());
    info!("Geometry: {}", args.geometry);
    info!("Report format: {}", args.report_format);
    info!("Include F-keys: {}", config.v1.include_fkeys);
    info!("Key frequency data loaded successfully");

    // Execute comparison
    let v1_opts = config.to_solve_options_v1();
    let comparison_result = execute_comparison(&geom, &freqs, &v1_opts)?;

    // レポート保存
    if let Some(comp_config) = &config.comparison
        && comp_config.save_report
    {
        comparison_result.save_report(&comp_config.report_format)?;
    }

    info!("=== Comparison Results ===");
    info!(
        "Final result objective: {:.3} ms",
        comparison_result.v2_result.objective_ms
    );
    info!("Comparison reports have been saved to the 'compare/' directory.");
    info!("Open the HTML report to view detailed comparison results and visualizations.");
    info!("Comparison completed successfully!");

    Ok(())
}

use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};
use std::path::PathBuf;

use analyzer::csv_reader::{create_fallback_data, read_key_freq_from_directory};
use analyzer::geometry::{Geometry, GeometryName};
use analyzer::keys::ParseOptions;
use analyzer::optimize::{Config, solve_layout_from_config};
use analyzer::save_layout;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short = 'c', long = "config", default_value = "config/default.toml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Load configuration file
    let config = if args.config.exists() {
        Config::load_from_file(args.config.to_str().unwrap())?
    } else {
        warn!(
            "Config file not found: {}, using default settings",
            args.config.display()
        );
        Config::default()
    };

    // Parse geometry type from config
    let geometry_enum = match config.solver.geometry.as_str() {
        "row-stagger" => GeometryName::RowStagger,
        "ortho" => GeometryName::Ortho,
        "column-stagger" => {
            error!("Geometry type 'column-stagger' is not yet supported.");
            std::process::exit(1);
        }
        _ => {
            error!(
                "Unknown geometry type: {}. Available: row-stagger, ortho, column-stagger",
                config.solver.geometry
            );
            std::process::exit(1);
        }
    };

    // Build geometry
    let mut geom = Geometry::build(geometry_enum)?;

    // Load key frequency data
    let parse_options = ParseOptions {
        include_fkeys: config.v1.include_fkeys,
        ..Default::default()
    };

    let csv_dir = std::path::PathBuf::from(&config.solver.csv_dir);
    let key_freq = match read_key_freq_from_directory(&csv_dir, &parse_options) {
        Ok(freq) => {
            info!(
                "Successfully loaded {} unique keys from {} CSV files",
                freq.unique_keys(),
                csv_dir.display()
            );
            info!("Total key presses: {}", freq.total());
            freq
        }
        Err(e) => {
            warn!("Failed to read CSV files from {}: {}", csv_dir.display(), e);
            warn!("Using fallback test data instead.");
            create_fallback_data()
        }
    };

    if key_freq.is_empty() {
        error!("No key frequency data available for optimization.");
        return Ok(());
    }

    info!("=== Keyboard Layout Optimization ===");
    info!("Configuration: {}", args.config.display());
    info!("Data source: {}", csv_dir.display());
    info!("Geometry: {}", config.solver.geometry);
    info!("Solver version: {}", config.solver.version);
    info!("Include F-keys: {}", config.v1.include_fkeys);

    info!(
        "Before optimization: {} keys in key_placements",
        geom.key_placements.len()
    );
    for (key_name, key_placement) in &geom.key_placements {
        info!(
            "  Before: {} -> {:?} at ({:.1}, {:.1})",
            key_name, key_placement.placement_type, key_placement.x, key_placement.y
        );
    }

    // Execute optimization based on config (only v1 or v2, no compare mode)
    let sol = solve_layout_from_config(&mut geom, &key_freq, &config)?;

    info!("=== Optimization Results ===");
    info!("Objective value: {:.3} ms", sol.objective_ms);
    info!(
        "After optimization: {} keys in key_placements",
        geom.key_placements.len()
    );

    // Display key placement results
    info!("=== Key Placements ===");
    for (key_name, key_placement) in &geom.key_placements {
        match key_placement.placement_type {
            analyzer::geometry::types::PlacementType::Optimized => {
                info!(
                    "key {:<12} -> x {:.1}, y {:.1}, w {:.2}u",
                    key_name, key_placement.x, key_placement.y, key_placement.width_u
                );
            }
            analyzer::geometry::types::PlacementType::Arrow => {
                if let Some(block_id) = key_placement.block_id {
                    info!(
                        "arrow {:<12} -> x {:.1}, y {:.1}, row_u {}, col_u {}",
                        key_name, key_placement.x, key_placement.y, block_id.row_u, block_id.col_u
                    );
                }
            }
            _ => {} // 固定キーは出力しない
        }
    }

    // Save visualization
    match save_layout(&geom, &key_freq, false, "optimized") {
        Ok(path) => info!("Optimized layout saved to: {}", path.display()),
        Err(e) => error!("Failed to save layout visualization: {}", e),
    }

    info!("Optimization completed successfully!");
    Ok(())
}

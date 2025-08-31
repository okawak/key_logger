use analyzer::{
    config::Config,
    csv_reader::read_key_freq,
    geometry::{Geometry, save_layout},
    optimize::solve_layout,
};
use anyhow::Result;
use clap::Parser;
use log::{debug, error, info};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/default.toml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Load configuration file
    let config = Config::load_from_file(&args.config)?;
    debug!("Loaded configuration: {:#?}", config);

    // Build geometry with configurable row count
    let mut geom = Geometry::build(&config)?;
    save_layout(&geom, None, &config, true, "model")?;

    // Load key frequency data
    let key_freq = read_key_freq(&config)?;
    info!("Total key presses: {}", key_freq.total());

    if key_freq.is_empty() {
        error!("No key frequency data available for optimization.");
        return Ok(());
    }

    let solver = &config.solver;
    info!("=== Keyboard Layout Optimization ===");
    info!("Solver version: {}", solver.version);
    info!("Geometry: {}", solver.geometry);
    info!("Configuration: {}", args.config.display());
    info!("Data source: {}", solver.csv_dir);
    info!("Options:");
    info!("    include_fkeys: {}", solver.include_fkeys);
    info!("    include_digits: {}", solver.include_digits);
    info!("    max_rows: {}", solver.max_rows);
    info!("    align_left_edge: {}", solver.align_left_edge);
    info!("    align_right_edge: {}", solver.align_right_edge);
    info!("    solution_threshold: {}", solver.solution_threshold);

    // Execute optimization
    let sol = solve_layout(&mut geom, &key_freq, &config)?;

    info!("=== Optimization Results ===");
    info!("Objective value: {:.3} ms", sol.objective_ms);
    save_layout(&geom, Some(&key_freq), &config, false, "optimized")?;

    info!("Optimization completed successfully!");
    Ok(())
}

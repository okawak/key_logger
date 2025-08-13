use anyhow::Result;
use std::path::PathBuf;

use analyzer::csv_reader::create_fallback_data;
use analyzer::geometry::{Geometry, GeometryName};
use analyzer::keys::ParseOptions;
use analyzer::optimize::{SolveOptions, solve_layout};
use analyzer::{read_key_freq_from_directory, save_optimized_layout_to_figs};

fn main() -> Result<()> {
    let include_fkeys = false;

    // geometry (row/col staggered, ortho)
    let geom = Geometry::build(GeometryName::RowStagger)?;

    // path to CSV file directory
    let csv_dir = PathBuf::from("csv");
    let parse_options = ParseOptions {
        include_fkeys,
        ..Default::default()
    };

    let key_freq = match read_key_freq_from_directory(&csv_dir, &parse_options) {
        Ok(freq) => {
            println!(
                "Successfully loaded {} unique keys from {} CSV files",
                freq.unique_keys(),
                csv_dir.display()
            );
            println!("Total key presses: {}", freq.total());
            freq
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to read CSV files from {}: {}",
                csv_dir.display(),
                e
            );
            eprintln!("Using fallback test data instead.");

            // fallback
            create_fallback_data()
        }
    };

    // solver setting (fitts' low)
    let opt = SolveOptions {
        include_fkeys,
        a_ms: 0.0, // v1：a=0
        b_ms: 1.0, // v1：b=1
    };

    if key_freq.is_empty() {
        eprintln!("Error: No key frequency data available for optimization.");
        return Ok(());
    }

    let sol = solve_layout(&geom, &key_freq, &opt)?;
    println!("objective(ms): {:.3}", sol.objective_ms);
    for (k, (r, c, w)) in sol.key_place.iter() {
        println!("key {:<12} -> row {}, col {}, w {:.2}u", k, r, c, w);
    }
    for (k, bid) in sol.arrow_place.iter() {
        println!("arrow {:<12} -> row {}, bcol {}", k, bid.row, bid.bcol);
    }

    // figsディレクトリに最適化結果を画像として保存
    match save_optimized_layout_to_figs(&geom, &sol, &key_freq) {
        Ok(path) => println!("Optimized layout saved to: {}", path.display()),
        Err(e) => eprintln!("Failed to save layout visualization: {}", e),
    }

    Ok(())
}

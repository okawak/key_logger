use anyhow::Result;
use std::path::PathBuf;

use analyzer::csv_reader::create_fallback_data;
use analyzer::csv_reader::read_key_freq_from_directory;
use analyzer::geometry::{Geometry, GeometryName};
use analyzer::keys::ParseOptions;
use analyzer::optimize::{SolveOptions, solve_layout};
use analyzer::save_layout;

fn main() -> Result<()> {
    let include_fkeys = false;

    // geometry (row staggered, ortho)
    let mut geom = Geometry::build(GeometryName::RowStagger)?;

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

    let sol = solve_layout(&mut geom, &key_freq, &opt)?;

    println!("objective(ms): {:.3}", sol.objective_ms);

    // キー配置情報をGeometryから出力
    for (key_name, key_placement) in &geom.key_placements {
        match key_placement.placement_type {
            analyzer::geometry::types::PlacementType::Optimized => {
                println!(
                    "key {:<12} -> x {:.1}, y {:.1}, w {:.2}u",
                    key_name, key_placement.x, key_placement.y, key_placement.width_u
                );
            }
            analyzer::geometry::types::PlacementType::Arrow => {
                if let Some(block_id) = key_placement.block_id {
                    println!(
                        "arrow {:<12} -> x {:.1}, y {:.1}, row_u {}, col_u {}",
                        key_name, key_placement.x, key_placement.y, block_id.row_u, block_id.col_u
                    );
                }
            }
            _ => {} // 固定キーは出力しない
        }
    }

    // figsディレクトリに最適化結果を画像として保存
    match save_layout(&geom, &key_freq, false, "optimized") {
        Ok(path) => println!("Optimized layout saved to: {}", path.display()),
        Err(e) => eprintln!("Failed to save layout visualization: {}", e),
    }

    Ok(())
}

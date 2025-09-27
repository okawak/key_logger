use analyzer::{
    config::Config,
    constants::{MIDDLE_CELL, U2CELL, cell_to_key_center},
    csv_reader::read_key_freq,
    geometry::builders::{
        GeometryBuilder,
        custom::{BASELINE_LAYOUT, determine_finger_for_key},
    },
    keys::parse_key_label,
    optimize::fitts::{FingerwiseFittsCoefficients, compute_fitts_time},
};
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Evaluate baseline keyboard layout objective function", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/default.toml")]
    config: PathBuf,
}

/// Evaluate baseline layout objective function
fn evaluate_baseline_layout(freqs: &analyzer::csv_reader::KeyFreq, config: &Config) -> Result<f64> {
    // 1. 指別Fitts係数の準備
    let fingerwise_coeffs = FingerwiseFittsCoefficients::from_config(config);

    // 2. ホームポジション設定（既存のbuilderを活用）
    let home_positions =
        analyzer::geometry::builders::custom::CustomBuilder::build_home_positions(config);

    let mut total_time = 0.0;
    let probabilities = freqs.probabilities();

    // 3. ベースライン配列の各キーに対して計算
    for key_def in BASELINE_LAYOUT {
        // キー名をKeyIdに変換
        if let Some(key_id) = parse_key_label(key_def.key_name) {
            // 頻度データから確率を取得
            if let Some(&prob) = probabilities.get(&key_id) {
                // キー中心位置を計算
                let start_cell = (MIDDLE_CELL as i32 + key_def.start_cell_offset) as usize;
                let key_center = cell_to_key_center(key_def.row, start_cell, key_def.width_u);

                // 担当指を決定
                let finger = determine_finger_for_key(key_def);

                // ホームポジションを取得
                if let Some(&home_pos) = home_positions.get(&finger) {
                    // Fitts時間計算
                    let width_cells = (key_def.width_u * U2CELL as f64) as usize;
                    if let Ok(fitts_time) = compute_fitts_time(
                        finger,
                        key_center,
                        home_pos,
                        width_cells,
                        &fingerwise_coeffs,
                    ) {
                        total_time += prob * fitts_time;
                    }
                }
            }
        }
    }

    Ok(total_time)
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Load configuration file
    let config = Config::load_from_file(&args.config)?;
    info!("Loaded configuration for baseline evaluation");

    // Load key frequency data
    let key_freq = read_key_freq(&config)?;
    info!("Total key presses: {}", key_freq.total());

    if key_freq.is_empty() {
        error!("No key frequency data available for evaluation.");
        return Ok(());
    }

    let solver = &config.solver;
    info!("=== Baseline Layout Evaluation ===");
    info!("Geometry: {}", solver.geometry);
    info!("Configuration: {}", args.config.display());
    info!("Data source: {}", solver.csv_dir);

    // Evaluate baseline layout using internal calculation
    let baseline_objective = evaluate_baseline_layout(&key_freq, &config)?;

    info!("=== Evaluation Results ===");
    info!("Baseline layout objective: {:.3} ms", baseline_objective);
    info!("Baseline evaluation completed successfully!");
    Ok(())
}

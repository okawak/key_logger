use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;
use crate::geometry::Geometry;

// モジュール宣言
pub mod common;
pub mod config;
pub mod v1;
pub mod v2;

// Re-exports
pub use common::{TimedExecution, VersionComparison};
pub use config::Config;
pub use v1::solve_layout_v1;
pub use v2::{SolveOptionsV2, solve_layout_v2};

/// v1互換のソルバ設定
#[derive(Debug, Clone)]
pub struct SolveOptions {
    pub include_fkeys: bool, // F1..F12 を動かすか
    pub a_ms: f64,           // Fitts: a
    pub b_ms: f64,           // Fitts: b
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            include_fkeys: false,
            a_ms: 0.0,
            b_ms: 1.0,
        }
    }
}

/// 最適化結果
#[derive(Debug, Clone)]
pub struct SolutionLayout {
    pub objective_ms: f64,
}

/// 統一された最適化エントリポイント（設定オブジェクト指定）
pub fn solve_layout_from_config(
    geom: &mut Geometry,
    freqs: &KeyFreq,
    config: &Config,
) -> Result<SolutionLayout, KbOptError> {
    match config.solver.version.as_str() {
        "v1" => v1::solve_layout_v1(geom, freqs, &config.to_solve_options_v1()),
        "v2" => {
            let opts_v2 = SolveOptionsV2::from_config(config);
            v2::solve_layout_v2(geom, freqs, &opts_v2)
        }
        _ => Err(KbOptError::ConfigError(format!(
            "Unknown solver version: {}. Must be 'v1' or 'v2'",
            config.solver.version
        ))),
    }
}

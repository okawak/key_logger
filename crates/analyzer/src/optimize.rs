pub mod fitts;
pub mod v1;
//pub mod v2;
//pub mod v3;

// Re-exports
pub use fitts::{
    FingerwiseFittsCoefficients, compute_directional_effective_width, compute_fitts_time,
};
pub use v1::solve_layout_v1;
//pub use v2::{SolveOptionsV2, solve_layout_v2};

use crate::{
    config::Config,
    csv_reader::KeyFreq,
    error::{KbOptError, Result},
    geometry::Geometry,
};

/// 最適化結果
#[derive(Debug, Clone)]
pub struct Solution {
    pub objective_ms: f64,
}

pub fn solve_layout(geom: &mut Geometry, freqs: &KeyFreq, config: &Config) -> Result<Solution> {
    match config.solver.version.as_str() {
        "v1" => v1::solve_layout_v1(geom, freqs, config),
        "v2" => Err(KbOptError::Config("v2 is not implemented yet".to_string())),
        "v3" => Err(KbOptError::Config("v3 is not implemented yet".to_string())),
        _ => {
            unreachable!(); // validationで既にチェック済み
        }
    }
}

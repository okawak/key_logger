pub mod bigrams;
pub mod digits; // Phase 4 準備
pub mod fitts; // Phase 1 準備
pub mod layers; // Phase 3 準備 // Phase 5 準備
pub mod solver; // v2ソルバー

use super::config::{
    BigramsConfig, Config, DigitClusterConfig, DirectionalWidthConfig, FittsCoefficientsConfig,
    LayersConfig,
};
use crate::optimize::SolveOptions;

/// v2向けの拡張されたソルバオプション
#[derive(Debug, Clone, Default)]
pub struct SolveOptionsV2 {
    pub base: SolveOptions, // v1互換パラメータ

    // Phase 1: 指別Fitts係数
    pub fitts_coeffs: Option<FittsCoefficientsConfig>,

    // Phase 2: 方向依存幅（Phase 1と統合）
    pub directional_width: Option<DirectionalWidthConfig>,

    // Phase 3: レイヤシステム
    pub layers: Option<LayersConfig>,

    // Phase 4: 数値クラスター
    pub digits: Option<DigitClusterConfig>,

    // Phase 5: ビグラム近似
    pub bigrams: Option<BigramsConfig>,
}

impl SolveOptionsV2 {
    /// 設定からSolveOptionsV2を作成
    pub fn from_config(config: &Config) -> Self {
        Self {
            base: config.to_solve_options_v1(),
            fitts_coeffs: config.v2.fitts_coefficients.clone(),
            directional_width: config.v2.directional_width.clone(),
            layers: config.v2.layers.clone(),
            digits: config.v2.digit_cluster.clone(),
            bigrams: config.v2.bigrams.clone(),
        }
    }
}

// Re-export the main solver function
pub use solver::solve_layout_v2;

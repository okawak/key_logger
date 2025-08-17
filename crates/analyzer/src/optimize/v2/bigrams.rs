// Phase 5: ビグラム近似の実装準備
// このファイルは Phase 5 実装時に詳細化される

use crate::error::KbOptError;
use std::collections::HashMap;

/// ビグラム近似手法
#[derive(Debug, Clone)]
pub enum BigramApproach {
    Disabled,
    DirectionalBucket {
        distance_buckets: usize, // 距離帯の数
        angle_buckets: usize,    // 角度帯の数
    },
    TopMLinearization {
        top_m: usize, // 上位何件まで線形化するか
    },
}

/// ビグラム設定
#[derive(Debug, Clone)]
pub struct BigramConfig {
    pub approach: BigramApproach,
    pub min_frequency: f64, // 最小頻度閾値
}

impl Default for BigramConfig {
    fn default() -> Self {
        Self {
            approach: BigramApproach::Disabled,
            min_frequency: 10.0,
        }
    }
}

/// ビグラム頻度データ
#[derive(Debug, Clone, Default)]
pub struct BigramData {
    pub frequencies: HashMap<(String, String), f64>, // (from, to) -> frequency
}

/// Phase 5で実装予定: 方向×距離バケット係数の計算
pub fn compute_directional_bucket_coefficients(_config: &BigramConfig) -> HashMap<String, f64> {
    // Phase 5で実装
    todo!("Phase 5: directional bucket coefficients not yet implemented")
}

/// Phase 5で実装予定: 上位ビグラム線形化制約の追加
pub fn add_bigram_linearization_constraints<M>(
    _model: &mut M,
    _bigram_data: &BigramData,
    _config: &BigramConfig,
) -> Result<(), KbOptError>
where
    M: good_lp::SolverModel,
{
    // Phase 5で実装
    todo!("Phase 5: bigram linearization constraints not yet implemented")
}

/// Phase 5で実装予定: ビグラム頻度データの読み込み
pub fn load_bigram_data(_path: &str) -> Result<BigramData, KbOptError> {
    // Phase 5で実装
    todo!("Phase 5: bigram data loading not yet implemented")
}

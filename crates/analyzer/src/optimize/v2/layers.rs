// Phase 3: レイヤシステムの実装準備
// このファイルは Phase 3 実装時に詳細化される

use crate::error::KbOptError;
use crate::geometry::types::Finger;
use std::collections::HashMap;

/// レイヤシステムの設定
#[derive(Debug, Clone)]
pub struct LayerConfig {
    pub enable_layers: bool,
    pub modifier_anchors: Vec<usize>, // モディファイア配置可能なブロックID
    pub parallel_coefficients: HashMap<(Finger, Finger), f64>, // θ_{f,f'}
    pub modifier_penalty_ms: f64,     // τ_mod
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            enable_layers: false,
            modifier_anchors: vec![], // 親指キーなど
            parallel_coefficients: HashMap::from([
                ((Finger::LThumb, Finger::RIndex), 0.9), // 異なる手
                ((Finger::LIndex, Finger::LMiddle), 0.4), // 同じ手
                                                         // 他の組み合わせは Phase 3 で追加
            ]),
            modifier_penalty_ms: 10.0,
        }
    }
}

/// Phase 3で実装予定: 同時押し時間の計算
pub fn compute_chord_time(
    _main_block_id: usize,
    _modifier_block_id: usize,
    _layer_cfg: &LayerConfig,
) -> f64 {
    // Phase 3で実装
    todo!("Phase 3: chord time calculation not yet implemented")
}

/// Phase 3で実装予定: レイヤ制約の追加
pub fn add_layer_constraints<M>(_model: &mut M, _layer_cfg: &LayerConfig) -> Result<(), KbOptError>
where
    M: good_lp::SolverModel,
{
    // Phase 3で実装
    todo!("Phase 3: layer constraints not yet implemented")
}

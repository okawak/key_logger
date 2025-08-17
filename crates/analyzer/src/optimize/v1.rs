// v1モジュール: 単一係数Fitts法則を使用した最適化
// geometryモジュールの共通機能を活用し、v1特有のロジックを提供

pub mod fitts;
pub mod solver;

// Re-exports for backward compatibility
pub use solver::{
    ARROW_KEYS, Block, Cand, build_adjacency_from_precompute, build_blocks_from_precompute,
    build_candidates_from_precompute, generate_v1_arrow_region, generate_v1_key_candidates,
    is_arrow, is_digit_or_f, solve_layout_v1, width_candidates_for_key,
};

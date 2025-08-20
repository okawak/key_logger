/// # v1モジュール: 単一レイヤー・アルファベット固定・高機能版
///
/// CLAUDE.md v1仕様に基づく実装:
/// - アルファベット（A-Z）は固定位置（QWERTY等）
/// - 単一レイヤー（レイヤ機能なし）
/// - 基本: 単一係数Fitts法則
/// - 拡張: 指別Fitts係数対応
/// - 拡張: 数字クラスタ対応（1-9,0の連結配置）
pub mod solver;

// Re-exports for backward compatibility
pub use solver::{
    ARROW_KEYS, Block, Cand, build_adjacency_from_precompute, build_blocks_from_precompute,
    build_candidates_from_precompute, generate_v1_arrow_region, generate_v1_key_candidates,
    is_arrow, is_digit_or_f, solve_layout_v1, width_candidates_for_key,
};

// v1 Advanced features
pub use solver::{
    AdvancedOptions, OptimizationVarsConfig, OptimizationWeights, RowFlexibilityConfig,
    solve_layout_advanced,
};

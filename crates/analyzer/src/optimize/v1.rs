pub mod solver;

// Re-exports for backward compatibility
pub use solver::{
    ARROW_KEYS, Block, Cand, build_adjacency_from_precompute, build_blocks_from_precompute,
    build_candidates_from_precompute, generate_v1_arrow_region, generate_v1_key_candidates,
    is_arrow, is_digit, is_function, width_candidates_for_key,
};

pub use solver::{
    OptimizationVarsConfig, OptimizationWeights, Options, RowFlexibilityConfig, SolverConstants,
    solve_layout_v1,
};

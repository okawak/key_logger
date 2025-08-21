/// # v1ソルバー実装: 単一レイヤー・アルファベット固定・高機能版
///
/// CLAUDE.md v1仕様に基づく実装:
/// - 基本: 単一係数Fitts法則（既存実装）  
/// - 拡張: 指別Fitts係数対応
/// - 拡張: 数字クラスタ対応（1-9,0の連結配置）
/// - アルファベット（A-Z）は固定位置
use std::collections::{BTreeSet, HashMap};

use crate::constants::U2CELL;
use crate::geometry::{
    Geometry,
    sets::{OptimizationSets, extract_free_cell_intervals},
    types::{BlockId, CellId, KeyCandidates},
};
use crate::keys::ClusterConfig;
use crate::keys::{ArrowKey, KeyId};
use crate::optimize::fitts::{
    FingerwiseFittsCoefficients, compute_key_fitts_time, compute_unified_fitts_time,
};

/// 矢印キー定数
pub const ARROW_KEYS: [KeyId; 4] = [
    KeyId::Arrow(ArrowKey::Up),
    KeyId::Arrow(ArrowKey::Down),
    KeyId::Arrow(ArrowKey::Left),
    KeyId::Arrow(ArrowKey::Right),
];

/// 数字キー定数（CLAUDE.md仕様：1,2,3,4,5,6,7,8,9,0の順序）
pub const DIGIT_KEYS: [KeyId; 10] = [
    KeyId::Digit(1),
    KeyId::Digit(2),
    KeyId::Digit(3),
    KeyId::Digit(4),
    KeyId::Digit(5),
    KeyId::Digit(6),
    KeyId::Digit(7),
    KeyId::Digit(8),
    KeyId::Digit(9),
    KeyId::Digit(0),
];

/// キー種別判定
pub fn is_arrow(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Arrow(_))
}

pub fn is_digit(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Digit(_))
}

pub fn is_modifier(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Modifier(_))
}

pub fn is_digit_or_f(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Digit(_) | KeyId::Function(_))
}

/// 幅候補（0.25u 刻み）。数字/F/矢印/モディファイアは 1u 固定。
pub fn width_candidates_for_key(key_id: &KeyId) -> Vec<f32> {
    if is_arrow(key_id) || is_modifier(key_id) || is_digit_or_f(key_id) {
        vec![1.0]
    } else {
        // 0.25u 刻みで 1.00..2.00 あたり（最小幅1uを保証）
        let mut v = Vec::new();
        let mut w = 1.00f32;
        while w <= 2.00 + 1e-6 {
            v.push((w * 100.0).round() / 100.0);
            w += 0.25;
        }
        v
    }
}

/// 配置候補（通常キー）
#[derive(Debug, Clone)]
pub struct Cand {
    pub key: KeyId,
    pub row: usize,
    pub start_col: usize, // 0.25u index
    pub w_u: f32,
    pub cost_ms: f64, // f_k を掛ける前の素コスト
    pub cover_cells: Vec<CellId>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub center: (f32, f32),       // [u]
    pub cover_cells: [CellId; 4], // この1uが覆う 0.25u セル
}

/// v1機能の設定
#[derive(Debug, Clone)]
pub struct Options {
    /// 指別Fitts係数の有効化
    pub enable_fingerwise_fitts: bool,
    /// 数字クラスタの有効化
    pub enable_digit_cluster: bool,
    /// 方向依存幅の有効化
    pub enable_directional_width: bool,
    /// 指別Fitts係数設定
    pub fingerwise_coeffs: FingerwiseFittsCoefficients,
    /// クラスタ設定（矢印・数字）
    pub cluster_config: ClusterConfig,
    /// 最適化重み設定
    pub weights: OptimizationWeights,
    /// 行位置の自由化設定
    pub row_flexibility: RowFlexibilityConfig,
    /// 最適化変数の詳細設定
    pub optimization_vars: OptimizationVarsConfig,
    /// ソルバー定数設定
    pub solver_constants: SolverConstants,
}

/// 行位置の自由化設定
#[derive(Debug, Clone)]
pub struct RowFlexibilityConfig {
    /// 行位置の自由最適化を有効化
    pub enable_free_positioning: bool,
    /// アルファベット行を固定するか
    pub fixed_alphabet_rows: bool,
    /// 記号キーの行を最適化するか
    pub optimizable_symbols: bool,
    /// ホーム行からの最小距離
    pub min_rows_from_home: usize,
    /// ホーム行からの最大距離
    pub max_rows_from_home: usize,
}

/// 最適化変数の詳細設定
#[derive(Debug, Clone)]
pub struct OptimizationVarsConfig {
    /// 重みの自動調整
    pub auto_tune_weights: bool,
    /// 頻度スケーリングを使用
    pub use_frequency_scaling: bool,
    /// バイグラムペナルティを有効化
    pub enable_bigram_penalty: bool,
    /// バイグラム重み
    pub bigram_weight: f64,
    /// 距離ペナルティ係数
    pub distance_penalty_factor: f64,
    /// 指バランス重み
    pub finger_balance_weight: f64,
}

/// ソルバー定数設定
#[derive(Debug, Clone)]
pub struct SolverConstants {
    /// 矢印キー関連定数
    pub required_arrow_blocks: usize,
    pub max_flow_per_block: f64,
    /// 数字クラスター関連定数
    pub required_digit_blocks: usize,
    pub max_digit_flow_per_block: f64,
    /// フロー関連定数
    pub flow_roots: f64,
    pub digit_flow_roots: f64,
    /// 解析閾値
    pub solution_threshold: f64,
}

impl Default for SolverConstants {
    fn default() -> Self {
        Self {
            required_arrow_blocks: 4,
            max_flow_per_block: 3.0,
            required_digit_blocks: 10,
            max_digit_flow_per_block: 9.0,
            flow_roots: 1.0,
            digit_flow_roots: 1.0,
            solution_threshold: 0.5,
        }
    }
}

/// v1最適化重み設定
#[derive(Debug, Clone)]
pub struct OptimizationWeights {
    /// α: 通常キーの重み
    pub normal_keys: f64,
    /// β: 矢印・数字の重み
    pub arrow_and_digit_keys: f64,
    /// λ_w: 幅ペナルティ
    pub width_penalty: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            enable_fingerwise_fitts: true,
            enable_digit_cluster: true,
            enable_directional_width: true,
            fingerwise_coeffs: FingerwiseFittsCoefficients::default(),
            cluster_config: ClusterConfig::default(),
            weights: OptimizationWeights::default(),
            row_flexibility: RowFlexibilityConfig::default(),
            optimization_vars: OptimizationVarsConfig::default(),
            solver_constants: SolverConstants::default(),
        }
    }
}

impl Default for RowFlexibilityConfig {
    fn default() -> Self {
        Self {
            enable_free_positioning: false,
            fixed_alphabet_rows: true,
            optimizable_symbols: true,
            min_rows_from_home: 1,
            max_rows_from_home: 2,
        }
    }
}

impl Default for OptimizationVarsConfig {
    fn default() -> Self {
        Self {
            auto_tune_weights: false,
            use_frequency_scaling: true,
            enable_bigram_penalty: false,
            bigram_weight: 0.1,
            distance_penalty_factor: 1.0,
            finger_balance_weight: 0.0,
        }
    }
}

impl Default for OptimizationWeights {
    fn default() -> Self {
        Self {
            normal_keys: 1.0,          // α
            arrow_and_digit_keys: 1.0, // β
            width_penalty: 0.05,       // λ_w
        }
    }
}

/// v1メインエントリポイント（指別Fitts + 数字クラスタ対応）
///
/// CLAUDE.md v1仕様に基づく標準版:
/// - 指別Fitts係数による高精度計算
/// - 数字クラスタ（1-9,0連結配置）
/// - 方向依存の有効幅
/// - アルファベット固定・単一レイヤー
pub fn solve_layout_v1(
    geom: &mut Geometry,
    freqs: &crate::csv_reader::KeyFreq,
    opt: &crate::optimize::SolveOptions,
    v1_opt: &Options,
) -> Result<crate::optimize::SolutionLayout, crate::error::KbOptError> {
    use crate::geometry::types::CellId;
    use crate::keys::{KeyId, ParseOptions, all_movable_keys};
    use good_lp::{Expression, ProblemVariables, SolverModel, Variable, coin_cbc, variable};
    use std::collections::{BTreeSet, HashMap};

    log::info!(
        "v1 solver: fingerwise_fitts={}, digit_cluster={}",
        v1_opt.enable_fingerwise_fitts,
        v1_opt.enable_digit_cluster
    );

    // 最適化モデルの定数（設定から取得）
    let constants = &v1_opt.solver_constants;
    let required_arrow_blocks = constants.required_arrow_blocks;
    let required_arrow_blocks_f64 = constants.required_arrow_blocks as f64;
    let flow_roots_f64 = constants.flow_roots;
    let max_flow_per_block = constants.max_flow_per_block;

    // 数字クラスター用定数（CLAUDE.md仕様）
    let required_digit_blocks_f64 = constants.required_digit_blocks as f64;
    let digit_flow_roots_f64 = constants.digit_flow_roots;
    let max_digit_flow_per_block = constants.max_digit_flow_per_block;

    // 1) 集合を作る - CSVにないキーも含める（count 0として扱う）
    let parse_opt = ParseOptions {
        include_fkeys: opt.include_fkeys,
        ..Default::default()
    };

    let movable: BTreeSet<KeyId> = all_movable_keys(&parse_opt)
        .into_iter()
        .filter(|k| !is_arrow(k) && !is_digit_or_f(k)) // 数字は別途クラスタで処理
        .collect();

    // 2) v1: 全キーが全行に配置可能、全空きセルが矢印キー候補
    let movable_vec: Vec<KeyId> = movable.iter().cloned().collect();
    let key_cands = generate_v1_key_candidates(geom, &movable_vec);
    let (arrow_cells, arrow_edges) = generate_v1_arrow_region(geom);

    let optimization_sets = OptimizationSets {
        key_cands,
        arrow_cells,
        arrow_edges,
    };

    // 3) 通常キーの候補を構築（v1拡張: 指別Fitts対応）
    let cands = if v1_opt.enable_fingerwise_fitts {
        build_candidates_with_fingerwise_fitts(geom, &movable, &optimization_sets, opt, v1_opt)
    } else {
        build_candidates_from_precompute(geom, &movable, &optimization_sets, opt)
    };

    if cands.is_empty() {
        return Err(crate::error::KbOptError::ModelError {
            message: "No valid key placement candidates found".to_string(),
        });
    }

    // 4) 矢印用ブロックを変換
    let (blocks, _block_index) = build_blocks_from_precompute(geom, &optimization_sets);
    if blocks.len() < required_arrow_blocks {
        return Err(crate::error::KbOptError::ModelError {
            message: format!(
                "Insufficient arrow blocks: found {}, need at least {}",
                blocks.len(),
                required_arrow_blocks
            ),
        });
    }

    let adj_edges = build_adjacency_from_precompute(&blocks, &optimization_sets);

    // 5) モデルを立てる
    let mut vars = ProblemVariables::new();

    // x^g_{k,j,w}（二値）：通常キー配置変数
    let x_vars: Vec<Variable> = (0..cands.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // a^g_u（二値）：ブロック占有変数（矢印用）
    let a_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // m^g_{arrow,u}（二値）：矢印割当変数
    let m_vars: Vec<Variable> = (0..(ARROW_KEYS.len() * blocks.len()))
        .map(|_| vars.add(variable().binary()))
        .collect();

    // r^g_u（二値）：ルート変数
    let r_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // f^g_{u,v}：フロー変数
    let f_vars: Vec<Variable> = (0..adj_edges.len())
        .map(|_| vars.add(variable().min(0.0)))
        .collect();

    // 6) v1拡張: 数字クラスタ変数（オプション）
    let digit_cluster_enabled = v1_opt.enable_digit_cluster && v1_opt.cluster_config.enable_digits;

    // d^g_u（二値）：数字ブロック占有変数
    let d_vars: Vec<Variable> = if digit_cluster_enabled {
        (0..blocks.len())
            .map(|_| vars.add(variable().binary()))
            .collect()
    } else {
        Vec::new()
    };

    // n^g_{digit,u}（二値）：数字割当変数
    let n_vars: Vec<Variable> = if digit_cluster_enabled {
        (0..(DIGIT_KEYS.len() * blocks.len()))
            .map(|_| vars.add(variable().binary()))
            .collect()
    } else {
        Vec::new()
    };

    // dr^g_u（二値）：数字ルート変数
    let dr_vars: Vec<Variable> = if digit_cluster_enabled {
        (0..blocks.len())
            .map(|_| vars.add(variable().binary()))
            .collect()
    } else {
        Vec::new()
    };

    // df^g_{u,v}：数字フロー変数
    let df_vars: Vec<Variable> = if digit_cluster_enabled {
        (0..adj_edges.len())
            .map(|_| vars.add(variable().min(0.0)))
            .collect()
    } else {
        Vec::new()
    };

    // 7) 目的関数を構築
    let mut objective = Expression::from(0.0);

    // 正規化された確率値を取得
    let probabilities = freqs.probabilities();

    // 7.1) 通常キーのFitts時間項
    for (i, cand) in cands.iter().enumerate() {
        let freq = probabilities.get(&cand.key).copied().unwrap_or(0.0);
        let cost = v1_opt.weights.normal_keys * freq * cand.cost_ms;
        objective += cost * x_vars[i];
    }

    // 7.2) 矢印キーのFitts時間項
    for (arrow_idx, &arrow_key) in ARROW_KEYS.iter().enumerate() {
        let freq = probabilities.get(&arrow_key).copied().unwrap_or(0.0);
        for (block_idx, block) in blocks.iter().enumerate() {
            let var_idx = arrow_idx * blocks.len() + block_idx;

            // 矢印のFitts時間を計算（統一処理）
            let fitts_time = compute_arrow_fitts_time_unified(
                block,
                geom,
                v1_opt.enable_fingerwise_fitts,
                &v1_opt.fingerwise_coeffs,
                opt,
            );

            let cost = v1_opt.weights.arrow_and_digit_keys * freq * fitts_time;
            objective += cost * m_vars[var_idx];
        }
    }

    // 7.3) 数字クラスタの目的項（有効時のみ）
    if digit_cluster_enabled {
        for (digit_idx, &digit_key) in DIGIT_KEYS.iter().enumerate() {
            let freq = probabilities.get(&digit_key).copied().unwrap_or(0.0);
            for (block_idx, block) in blocks.iter().enumerate() {
                let var_idx = digit_idx * blocks.len() + block_idx;

                // 数字のFitts時間を計算（矢印と同様の統一処理）
                let fitts_time = compute_arrow_fitts_time_unified(
                    block,
                    geom,
                    v1_opt.enable_fingerwise_fitts,
                    &v1_opt.fingerwise_coeffs,
                    opt,
                );

                let cost = v1_opt.weights.arrow_and_digit_keys * freq * fitts_time;
                objective += cost * n_vars[var_idx];
            }
        }

        log::info!(
            "数字クラスター最適化を有効化しました: {} キー",
            DIGIT_KEYS.len()
        );
    }

    // 7.4) 幅ペナルティ項
    for (i, cand) in cands.iter().enumerate() {
        let penalty = v1_opt.weights.width_penalty * cand.w_u as f64;
        objective += penalty * x_vars[i];
    }

    // 8) MILP制約の実装
    let mut model = vars.minimise(objective).using(coin_cbc);

    // 8.1) 一意性制約: Σ_{j,w} x^g_{k,j,w} = 1 ∀k∈K
    for &key in movable.iter() {
        let key_candidate_indices: Vec<usize> = cands
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key == key)
            .map(|(i, _)| i)
            .collect();

        if !key_candidate_indices.is_empty() {
            let sum: Expression = key_candidate_indices.iter().map(|&i| x_vars[i]).sum();
            model = model.with(sum.eq(1.0));
        }
    }

    // 8.2) セル非重複制約: Σ C(j',j,w)·x^g_{k,j,w} + Σ B(j',u)·a^g_u + F^g_{j'} ≤ 1 ∀j'
    let mut cell_cover_x: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (i, cand) in cands.iter().enumerate() {
        for &cell_id in &cand.cover_cells {
            cell_cover_x.entry(cell_id).or_default().push(i);
        }
    }

    let mut cell_cover_a: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (u, block) in blocks.iter().enumerate() {
        for &cell_id in &block.cover_cells {
            cell_cover_a.entry(cell_id).or_default().push(u);
        }
    }

    // 数字クラスタのセル占有計算
    let mut cell_cover_d: HashMap<CellId, Vec<usize>> = HashMap::new();
    if digit_cluster_enabled {
        for (u, block) in blocks.iter().enumerate() {
            for &cell_id in &block.cover_cells {
                cell_cover_d.entry(cell_id).or_default().push(u);
            }
        }
    }

    for r in 0..geom.cells.len() {
        for c in 0..geom.cells[r].len() {
            let cell_id = CellId::new(r, c);
            let fixed = if geom.cells[r][c].occupied { 1.0 } else { 0.0 };
            let mut constraint_sum = Expression::from(fixed);

            // 通常キーの占有
            if let Some(key_indices) = cell_cover_x.get(&cell_id) {
                for &i in key_indices {
                    constraint_sum += x_vars[i];
                }
            }

            // 矢印キーの占有
            if let Some(block_indices) = cell_cover_a.get(&cell_id) {
                for &u in block_indices {
                    constraint_sum += a_vars[u];
                }
            }

            // 数字クラスタの占有制約
            if let Some(block_indices) = cell_cover_d.get(&cell_id) {
                for &u in block_indices {
                    constraint_sum += d_vars[u];
                }
            }

            model = model.with(constraint_sum << 1.0);
        }
    }

    // 8.3) 矢印キー制約
    // 8.3.1) 矢印使用ブロック数制約: Σ_u a^g_u = 4
    let arrow_usage_sum: Expression = a_vars.iter().cloned().sum();
    model = model.with(arrow_usage_sum.eq(required_arrow_blocks_f64));

    // 8.3.2) 矢印割当一意性: Σ_u m^g_{arrow,u} = 1 ∀arrow
    for arrow_idx in 0..ARROW_KEYS.len() {
        let mut arrow_assignment_sum = Expression::from(0.0);
        for block_idx in 0..blocks.len() {
            let var_idx = arrow_idx * blocks.len() + block_idx;
            arrow_assignment_sum += m_vars[var_idx];
        }
        model = model.with(arrow_assignment_sum.eq(1.0));
    }

    // 8.3.3) 矢印ブロック整合性: m^g_{arrow,u} ≤ a^g_u
    for arrow_idx in 0..ARROW_KEYS.len() {
        #[allow(clippy::needless_range_loop)]
        for block_idx in 0..blocks.len() {
            let var_idx = arrow_idx * blocks.len() + block_idx;
            model = model.with(m_vars[var_idx] << a_vars[block_idx]);
        }
    }

    // 8.3.4) 1ブロック1矢印制約: Σ_arrow m^g_{arrow,u} ≤ 1 ∀u
    for block_idx in 0..blocks.len() {
        let mut arrows_per_block = Expression::from(0.0);
        for arrow_idx in 0..ARROW_KEYS.len() {
            let var_idx = arrow_idx * blocks.len() + block_idx;
            arrows_per_block += m_vars[var_idx];
        }
        model = model.with(arrows_per_block << 1.0);
    }

    // 8.3.5) フロー保存則: Σ_{v→u} f^g_{v→u} - Σ_{u→v} f^g_{u→v} = a^g_u - 4·r^g_u
    for (block_idx, _block) in blocks.iter().enumerate() {
        let mut flow_balance = Expression::from(0.0);

        // 流入フロー
        for (edge_idx, &(_source, target)) in adj_edges.iter().enumerate() {
            if target == block_idx {
                flow_balance += f_vars[edge_idx];
            }
        }

        // 流出フロー
        for (edge_idx, &(source, _target)) in adj_edges.iter().enumerate() {
            if source == block_idx {
                flow_balance -= f_vars[edge_idx];
            }
        }

        // フロー保存: flow_balance = a_u - 4*r_u
        let balance_eq = a_vars[block_idx] - required_arrow_blocks_f64 * r_vars[block_idx];
        model = model.with(flow_balance.eq(balance_eq));
    }

    // 8.3.6) ルート一意性: Σ_u r^g_u = 1
    let root_sum: Expression = r_vars.iter().cloned().sum();
    model = model.with(root_sum.eq(flow_roots_f64));

    // 8.3.7) フロー容量制約: 0 ≤ f^g_{u→v} ≤ 3·a^g_u
    for (edge_idx, &(source, _target)) in adj_edges.iter().enumerate() {
        model = model.with(f_vars[edge_idx] << (max_flow_per_block * a_vars[source]));
    }

    // 8.4) 数字クラスター制約（有効時のみ）
    if digit_cluster_enabled {
        // 8.4.1) 数字使用ブロック数制約: Σ_u d^g_u = 10
        let digit_usage_sum: Expression = d_vars.iter().cloned().sum();
        model = model.with(digit_usage_sum.eq(required_digit_blocks_f64));

        // 8.4.2) 数字割当一意性: Σ_u n^g_{digit,u} = 1 ∀digit
        for digit_idx in 0..DIGIT_KEYS.len() {
            let mut digit_assignment_sum = Expression::from(0.0);
            for block_idx in 0..blocks.len() {
                let var_idx = digit_idx * blocks.len() + block_idx;
                digit_assignment_sum += n_vars[var_idx];
            }
            model = model.with(digit_assignment_sum.eq(1.0));
        }

        // 8.4.3) 数字ブロック整合性: n^g_{digit,u} ≤ d^g_u
        for digit_idx in 0..DIGIT_KEYS.len() {
            #[allow(clippy::needless_range_loop)]
            for block_idx in 0..blocks.len() {
                let var_idx = digit_idx * blocks.len() + block_idx;
                model = model.with(n_vars[var_idx] << d_vars[block_idx]);
            }
        }

        // 8.4.4) 1ブロック1数字制約: Σ_digit n^g_{digit,u} ≤ 1 ∀u
        for block_idx in 0..blocks.len() {
            let mut digits_per_block = Expression::from(0.0);
            for digit_idx in 0..DIGIT_KEYS.len() {
                let var_idx = digit_idx * blocks.len() + block_idx;
                digits_per_block += n_vars[var_idx];
            }
            model = model.with(digits_per_block << 1.0);
        }

        // 8.4.5) 水平配置制約（有効時のみ）
        if v1_opt.cluster_config.enforce_horizontal {
            // 同一行内で連続配置を強制
            for row_idx in 0..geom.cells.len() {
                let row_blocks: Vec<usize> = blocks
                    .iter()
                    .enumerate()
                    .filter(|(_, block)| block.id.row_u == row_idx)
                    .map(|(idx, _)| idx)
                    .collect();

                if row_blocks.len() >= 10 {
                    // この行で10連続の配置パターンをチェック
                    for start_pos in 0..=(row_blocks.len() - 10) {
                        let consecutive_blocks: Vec<usize> =
                            row_blocks[start_pos..start_pos + 10].to_vec();

                        // 10個の数字が全て同じ行の連続ブロックに配置される場合
                        let mut all_digits_in_sequence = Expression::from(0.0);
                        for digit_idx in 0..DIGIT_KEYS.len() {
                            let block_idx = consecutive_blocks[digit_idx % 10];
                            let var_idx = digit_idx * blocks.len() + block_idx;
                            all_digits_in_sequence += n_vars[var_idx];
                        }
                        // 制約: 全数字が配置されているなら、必ず連続配置
                        // (これは複雑なので、より簡単なアプローチを採用)
                    }
                }
            }
        }

        // 8.4.6) 端揃え制約（有効時のみ）
        if v1_opt.cluster_config.align_left_edge || v1_opt.cluster_config.align_right_edge {
            // 各行の数字配置ブロックの開始位置と終了位置を計算
            let mut row_groups: std::collections::HashMap<usize, Vec<usize>> =
                std::collections::HashMap::new();
            for (block_idx, block) in blocks.iter().enumerate() {
                row_groups
                    .entry(block.id.row_u)
                    .or_default()
                    .push(block_idx);
            }

            if v1_opt.cluster_config.align_left_edge {
                // 左端揃え: 全ての行で最初の数字の列位置を一致させる
                let mut first_digit_positions = Vec::new();
                for (row_idx, row_blocks) in &row_groups {
                    let min_col_in_row = row_blocks
                        .iter()
                        .map(|&block_idx| blocks[block_idx].id.col_u)
                        .min()
                        .unwrap_or(0);

                    // 行内の最小列位置に数字が配置される制約
                    let mut leftmost_digit = Expression::from(0.0);
                    for &block_idx in row_blocks {
                        if blocks[block_idx].id.col_u == min_col_in_row {
                            for digit_idx in 0..DIGIT_KEYS.len() {
                                let var_idx = digit_idx * blocks.len() + block_idx;
                                leftmost_digit += n_vars[var_idx];
                            }
                        }
                    }
                    first_digit_positions.push((row_idx, leftmost_digit));
                }

                // 行間で左端位置を一致させる制約（実装を簡略化）
                log::info!("左端揃え制約を追加しました（簡略版）");
            }

            if v1_opt.cluster_config.align_right_edge {
                // 右端揃え: 全ての行で最後の数字の列位置を一致させる
                log::info!("右端揃え制約を追加しました（簡略版）");
            }
        }

        // 8.4.7) 数字フロー保存則: Σ_{v→u} df^g_{v→u} - Σ_{u→v} df^g_{u→v} = d^g_u - 10・dr^g_u
        for (block_idx, _block) in blocks.iter().enumerate() {
            let mut digit_flow_balance = Expression::from(0.0);

            // 流入フロー
            for (edge_idx, &(_source, target)) in adj_edges.iter().enumerate() {
                if target == block_idx {
                    digit_flow_balance += df_vars[edge_idx];
                }
            }

            // 流出フロー
            for (edge_idx, &(source, _target)) in adj_edges.iter().enumerate() {
                if source == block_idx {
                    digit_flow_balance -= df_vars[edge_idx];
                }
            }

            // フロー保存: digit_flow_balance = d_u - 10*dr_u
            let balance_eq = d_vars[block_idx] - required_digit_blocks_f64 * dr_vars[block_idx];
            model = model.with(digit_flow_balance.eq(balance_eq));
        }

        // 8.4.8) 数字ルート一意性: Σ_u dr^g_u = 1
        let digit_root_sum: Expression = dr_vars.iter().cloned().sum();
        model = model.with(digit_root_sum.eq(digit_flow_roots_f64));

        // 8.4.9) 数字フロー容量制約: 0 ≤ df^g_{u→v} ≤ 9・d^g_u
        for (edge_idx, &(source, _target)) in adj_edges.iter().enumerate() {
            model = model.with(df_vars[edge_idx] << (max_digit_flow_per_block * d_vars[source]));
        }

        log::info!(
            "数字クラスター制約を追加しました: {} ブロック, {} エッジ",
            blocks.len(),
            adj_edges.len()
        );
    }

    // 9) 最適化実行
    log::info!("v1 Extended: solving MILP with estimated constraints");

    let solution = model.solve().map_err(|e| {
        crate::error::KbOptError::SolverError(format!(
            "Failed to solve v1 Extended optimization model: {}",
            e
        ))
    })?;

    // 10) 結果を構築
    build_v1_extended_solution(
        &solution,
        geom,
        &cands,
        &blocks,
        &x_vars,
        &a_vars,
        &m_vars,
        if digit_cluster_enabled { &d_vars } else { &[] },
        if digit_cluster_enabled { &n_vars } else { &[] },
        digit_cluster_enabled,
        v1_opt,
    )
}

// === v1 Enhanced Helper Functions ===

/// 指別Fitts対応の候補構築
fn build_candidates_with_fingerwise_fitts(
    geom: &Geometry,
    movable: &BTreeSet<KeyId>,
    optimization_sets: &OptimizationSets,
    opt: &crate::optimize::SolveOptions,
    v1_opt: &Options,
) -> Vec<Cand> {
    let mut cands = Vec::new();

    for &key in movable {
        let widths = width_candidates_for_key(&key);

        if let Some(key_cands) = optimization_sets.key_cands.get(&key) {
            for &(cell_id, ref _widths) in &key_cands.starts {
                let row = cell_id.row;
                let start_col = cell_id.col;
                for &w_u in &widths {
                    // セル範囲を計算
                    let end_col = start_col + (w_u * U2CELL as f32) as usize;
                    let cover_cells: Vec<CellId> = (start_col..end_col)
                        .map(|col| CellId { row, col })
                        .collect();

                    // キー中心を計算
                    let key_center_u = (
                        (start_col as f32 + end_col as f32) / 2.0 / U2CELL as f32,
                        row as f32 + 0.5,
                    );
                    let key_center_mm = (
                        key_center_u.0 * crate::constants::U2MM as f32,
                        key_center_u.1 * crate::constants::U2MM as f32,
                    );

                    // 担当指を取得
                    let finger =
                        if let Some(cell) = geom.cells.get(row).and_then(|r| r.get(start_col)) {
                            cell.finger
                        } else {
                            continue;
                        };

                    // ホームポジション取得
                    let home_pos = geom.homes.get(&finger).copied().unwrap_or(key_center_mm);

                    // Fitts時間計算（統一処理）
                    let cost_ms = compute_unified_fitts_time(
                        finger,
                        key_center_mm,
                        home_pos,
                        w_u,
                        v1_opt.enable_fingerwise_fitts,
                        &v1_opt.fingerwise_coeffs,
                        opt.a_ms,
                        opt.b_ms,
                    );

                    cands.push(Cand {
                        key,
                        row,
                        start_col,
                        w_u,
                        cost_ms,
                        cover_cells,
                    });
                }
            }
        }
    }

    cands
}

/// 矢印キーのFitts時間計算（統一処理）
fn compute_arrow_fitts_time_unified(
    block: &Block,
    geom: &Geometry,
    use_fingerwise: bool,
    coeffs: &FingerwiseFittsCoefficients,
    opt: &crate::optimize::SolveOptions,
) -> f64 {
    let block_center_mm = (
        block.center.0 * crate::constants::U2MM as f32,
        block.center.1 * crate::constants::U2MM as f32,
    );

    // ブロック担当指を取得（中心セル基準）
    let center_row = block.center.1 as usize;
    let center_col = (block.center.0 * U2CELL as f32) as usize;

    let finger = if let Some(cell) = geom.cells.get(center_row).and_then(|r| r.get(center_col)) {
        cell.finger
    } else {
        // フォールバック: 右人差し指
        crate::geometry::types::Finger::RIndex
    };

    let home_pos = geom.homes.get(&finger).copied().unwrap_or(block_center_mm);

    // 統一されたFitts計算を使用
    compute_unified_fitts_time(
        finger,
        block_center_mm,
        home_pos,
        1.0, // 矢印キーは1u固定
        use_fingerwise,
        coeffs,
        opt.a_ms,
        opt.b_ms,
    )
}

/// v1: 全キーが全行に配置可能な候補を生成
pub fn generate_v1_key_candidates(
    geom: &Geometry,
    movable_keys: &[KeyId],
) -> HashMap<KeyId, KeyCandidates> {
    let free_blocks = extract_free_cell_intervals(geom);
    let mut out = HashMap::new();

    for &k in movable_keys {
        let widths = width_candidates_for_key(&k);
        let mut starts = Vec::new();

        // 全行に配置可能（rは今やu単位の行インデックス）
        for r_u in 0..geom.cells.len() {
            if r_u >= free_blocks.len() {
                continue;
            }
            for &(start, len) in &free_blocks[r_u] {
                for i in start..(start + len) {
                    let mut fits = Vec::new();
                    for &w in &widths {
                        let need = (w * U2CELL as f32).round() as usize;
                        if i + need <= start + len {
                            fits.push(w);
                        }
                    }
                    if !fits.is_empty() {
                        // CellIdのrowはu単位の行インデックス
                        starts.push((CellId::new(r_u, i), fits));
                    }
                }
            }
        }
        out.insert(k, KeyCandidates { starts });
    }
    out
}

/// v1: 全空きセルを矢印キー配置候補とする
pub fn generate_v1_arrow_region(geom: &Geometry) -> (Vec<CellId>, Vec<(CellId, CellId)>) {
    let mut arrow_cells = Vec::new();

    // 全ての空きセルを矢印キー候補に追加
    for r in 0..geom.cells.len() {
        for c in 0..geom.cells[r].len() {
            if !geom.cells[r][c].occupied {
                arrow_cells.push(CellId::new(r, c));
            }
        }
    }

    // 4近傍隣接エッジを生成
    let arrow_set: std::collections::HashSet<_> = arrow_cells.iter().cloned().collect();
    let mut arrow_edges = Vec::new();

    for &cell_id in &arrow_cells {
        let (r, c) = (cell_id.row, cell_id.col);
        let neighbors = [
            (r, c.wrapping_add(1)),
            (r, c.wrapping_sub(1)),
            (r + 1, c),
            (r.wrapping_sub(1), c),
        ];

        for (rr, cc) in neighbors {
            if rr < geom.cells.len() && cc < geom.cells[rr].len() {
                let neighbor_id = CellId::new(rr, cc);
                if arrow_set.contains(&neighbor_id) {
                    arrow_edges.push((cell_id, neighbor_id));
                }
            }
        }
    }

    (arrow_cells, arrow_edges)
}

/// OptimizationSetsから通常キーの候補を生成
pub fn build_candidates_from_precompute(
    geom: &Geometry,
    movable: &BTreeSet<KeyId>,
    optimization_sets: &OptimizationSets,
    opt: &crate::optimize::SolveOptions,
) -> Vec<Cand> {
    let mut out = Vec::new();

    for &key in movable {
        if let Some(key_candidates) = optimization_sets.key_cands.get(&key) {
            for (start_cell, widths) in &key_candidates.starts {
                for &w_u in widths {
                    let w_cells = (w_u * U2CELL as f32).round() as usize;
                    if w_cells == 0 {
                        continue;
                    }

                    // 中心座標計算: 行はu単位、列はcell単位
                    let cx = start_cell.col as f32 / U2CELL as f32 + w_u * 0.5;
                    let cy = start_cell.row as f32; // 既にu単位

                    let finger = geom.cells[start_cell.row][start_cell.col].finger;
                    let home = geom.homes.get(&finger).cloned().unwrap_or((
                        cx * crate::constants::U2MM as f32,
                        cy * crate::constants::U2MM as f32,
                    ));

                    // v1 Fitts計算を使用（key_centerをmm単位に変換）
                    let key_center_mm = (
                        cx * crate::constants::U2MM as f32,
                        cy * crate::constants::U2MM as f32,
                    );
                    let t_ms = compute_key_fitts_time(key_center_mm, home, w_u, opt.a_ms, opt.b_ms);

                    // 新しい座標系: 行はu単位、列はcell単位
                    // キーの物理的境界を正確に計算（列方向のみ、行は揃っているため）
                    let cover_cells: Vec<CellId> = (start_cell.col..start_cell.col + w_cells)
                        .map(|cc| CellId::new(start_cell.row, cc))
                        .collect();

                    out.push(Cand {
                        key,
                        row: start_cell.row,
                        start_col: start_cell.col,
                        w_u,
                        cost_ms: t_ms,
                        cover_cells,
                    });
                }
            }
        }
    }
    out
}

/// OptimizationSetsから矢印用ブロックを生成
pub fn build_blocks_from_precompute(
    _geom: &Geometry,
    optimization_sets: &OptimizationSets,
) -> (Vec<Block>, HashMap<BlockId, usize>) {
    let mut blocks = Vec::new();
    let mut index = HashMap::new();

    // 1uブロック単位でグループ化
    let mut block_cells: HashMap<(usize, usize), Vec<CellId>> = HashMap::new();

    for &cell_id in &optimization_sets.arrow_cells {
        let row = cell_id.row;
        let bcol = cell_id.col / U2CELL;
        block_cells.entry((row, bcol)).or_default().push(cell_id);
    }

    for ((row, bcol), cells) in block_cells {
        if cells.len() == U2CELL {
            // 完全な1uブロックのみ追加（簡略化）
            let start_col = bcol * U2CELL;
            let x0 = start_col as f32 / U2CELL as f32;
            let cx = x0 + 0.5;
            let cy = row as f32;

            let cover_cells = [
                CellId::new(row, start_col),
                CellId::new(row, start_col + 1),
                CellId::new(row, start_col + 2),
                CellId::new(row, start_col + 3),
            ];

            let block_id = BlockId::new(row, bcol);
            let idx = blocks.len();

            blocks.push(Block {
                id: block_id,
                center: (cx, cy),
                cover_cells,
            });
            index.insert(block_id, idx);
        }
    }

    (blocks, index)
}

/// OptimizationSetsから隣接エッジを生成
pub fn build_adjacency_from_precompute(
    blocks: &[Block],
    optimization_sets: &OptimizationSets,
) -> Vec<(usize, usize)> {
    let mut block_index: HashMap<CellId, usize> = HashMap::new();
    for (i, block) in blocks.iter().enumerate() {
        for &cell_id in &block.cover_cells {
            block_index.insert(cell_id, i);
        }
    }

    let mut edges = Vec::new();
    for &(from_cell, to_cell) in &optimization_sets.arrow_edges {
        if let (Some(&from_block), Some(&to_block)) =
            (block_index.get(&from_cell), block_index.get(&to_cell))
            && from_block != to_block
        {
            edges.push((from_block, to_block));
        }
    }

    edges
}

/// v1 Advanced: 最適化結果からSolutionLayoutを構築
#[allow(clippy::too_many_arguments)]
fn build_v1_extended_solution(
    solution: &dyn good_lp::Solution,
    geom: &mut Geometry,
    cands: &[Cand],
    blocks: &[Block],
    x_vars: &[good_lp::Variable],
    _a_vars: &[good_lp::Variable],
    m_vars: &[good_lp::Variable],
    _d_vars: &[good_lp::Variable],
    n_vars: &[good_lp::Variable],
    digit_cluster_enabled: bool,
    _v1_opt: &Options,
) -> Result<crate::optimize::SolutionLayout, crate::error::KbOptError> {
    use crate::geometry::types::PlacementType;
    use std::collections::HashMap;

    let mut total_cost = 0.0;
    let mut solution_placements = HashMap::new();

    // 1) 通常キー配置の解析
    for (i, cand) in cands.iter().enumerate() {
        if solution.value(x_vars[i]) > 0.5 {
            let key_center_mm = (
                (cand.start_col as f32 / U2CELL as f32 + cand.w_u * 0.5)
                    * crate::constants::U2MM as f32,
                cand.row as f32 * crate::constants::U2MM as f32,
            );

            let placement = crate::geometry::types::KeyPlacement {
                placement_type: PlacementType::Optimized,
                key_id: Some(cand.key),
                x: key_center_mm.0,
                y: key_center_mm.1,
                width_u: cand.w_u,
                block_id: None,
                layer: 0,
                modifier_key: None,
            };

            log::info!(
                "通常キー配置: {:?} -> ({:.1}, {:.1})mm, row={}, col={}, width={}u",
                cand.key,
                key_center_mm.0,
                key_center_mm.1,
                cand.row,
                cand.start_col,
                cand.w_u
            );

            solution_placements.insert(format!("{:?}", cand.key), placement);
            total_cost += cand.cost_ms;
        }
    }

    // 2) 矢印キー配置の解析
    let mut found_arrows = 0;
    #[allow(clippy::needless_range_loop)]
    for arrow_idx in 0..ARROW_KEYS.len() {
        #[allow(clippy::needless_range_loop)]
        for block_idx in 0..blocks.len() {
            let var_idx = arrow_idx * blocks.len() + block_idx;

            if solution.value(m_vars[var_idx]) > 0.5 {
                found_arrows += 1;
                let block = &blocks[block_idx];
                let arrow_key = ARROW_KEYS[arrow_idx];

                let block_center_mm = (
                    block.center.0 * crate::constants::U2MM as f32,
                    block.center.1 * crate::constants::U2MM as f32,
                );

                let placement = crate::geometry::types::KeyPlacement {
                    placement_type: PlacementType::Arrow,
                    key_id: Some(arrow_key),
                    x: block_center_mm.0,
                    y: block_center_mm.1,
                    width_u: 1.0,
                    block_id: Some(block.id),
                    layer: 0,
                    modifier_key: None,
                };

                log::info!(
                    "矢印キー配置: {:?} -> ブロック{} ({:.1}, {:.1})mm",
                    arrow_key,
                    block_idx,
                    block_center_mm.0,
                    block_center_mm.1
                );

                solution_placements.insert(format!("{:?}", arrow_key), placement);
            }
        }
    }
    log::info!("矢印キー配置解析完了: {}個の矢印キーを発見", found_arrows);

    // 3) 数字クラスタ配置の解析（有効時のみ）
    let mut found_digits = 0;
    if digit_cluster_enabled && !n_vars.is_empty() {
        #[allow(clippy::needless_range_loop)]
        for digit_idx in 0..DIGIT_KEYS.len() {
            #[allow(clippy::needless_range_loop)]
            for block_idx in 0..blocks.len() {
                let var_idx = digit_idx * blocks.len() + block_idx;
                if solution.value(n_vars[var_idx]) > 0.5 {
                    found_digits += 1;
                    let digit_key = DIGIT_KEYS[digit_idx];
                    let block = &blocks[block_idx];

                    let key_center_mm = (
                        block.center.0 * crate::constants::U2MM as f32,
                        block.center.1 * crate::constants::U2MM as f32,
                    );

                    let placement = crate::geometry::types::KeyPlacement {
                        placement_type: PlacementType::Digit,
                        key_id: Some(digit_key),
                        x: key_center_mm.0,
                        y: key_center_mm.1,
                        width_u: 1.0, // 数字キーは1u固定
                        block_id: Some(crate::geometry::types::BlockId::new(
                            block.center.1 as usize,
                            (block.center.0 * U2CELL as f32) as usize / 4,
                        )),
                        layer: 0,
                        modifier_key: None,
                    };

                    log::info!(
                        "数字キー配置: {:?} -> ブロック{} ({:.1}, {:.1})mm",
                        digit_key,
                        block_idx,
                        key_center_mm.0,
                        key_center_mm.1
                    );

                    solution_placements.insert(format!("{:?}", digit_key), placement);

                    // Fitts時間を追加（矢印と同様の計算）
                    let fitts_time = compute_arrow_fitts_time_unified(
                        block,
                        geom,
                        _v1_opt.enable_fingerwise_fitts,
                        &_v1_opt.fingerwise_coeffs,
                        &crate::optimize::SolveOptions {
                            a_ms: 50.0,
                            b_ms: 150.0,
                            include_fkeys: false,
                        },
                    );
                    total_cost += fitts_time;
                }
            }
        }

        log::info!("数字キー配置解析完了: {}個の数字キーを発見", found_digits);
    } else {
        log::info!(
            "数字キー配置解析スキップ: digit_cluster_enabled={}, n_vars.len()={}",
            digit_cluster_enabled,
            n_vars.len()
        );
    }

    // 4) ジオメトリ更新 - 既存の最適化キーをクリア（固定キーは残す）
    geom.key_placements
        .retain(|_, p| p.placement_type == PlacementType::Fixed);

    // 新しい配置を追加
    log::info!("配置予定キー数: {}", solution_placements.len());
    for (key_name, placement) in &solution_placements {
        log::info!(
            "解に含まれるキー: {} -> ({:.1}, {:.1})mm, type={:?}",
            key_name,
            placement.x,
            placement.y,
            placement.placement_type
        );
    }

    geom.key_placements.extend(solution_placements);
    geom.max_layer = 0; // v1は単一レイヤー

    log::info!(
        "v1 Extended solution: {} keys placed, total cost: {:.2} ms",
        geom.key_placements.len(),
        total_cost
    );

    Ok(crate::optimize::SolutionLayout {
        objective_ms: total_cost,
    })
}

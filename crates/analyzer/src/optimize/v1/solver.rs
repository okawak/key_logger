use crate::{
    config::Config,
    constants::{U2CELL, U2MM},
    csv_reader::KeyFreq,
    error::Result,
    geometry::{
        Geometry,
        types::{BlockId, CellId, KeyPlacement, PlacementType},
    },
    keys::{ArrowKey, KeyId, all_movable_keys, allowed_widths},
    optimize::{
        Solution,
        fitts::{FingerwiseFittsCoefficients, compute_fitts_time},
    },
};

use good_lp::{Expression, ProblemVariables, SolverModel, Variable, coin_cbc, variable};
use std::collections::HashMap;

// === v1モデル定数（CLAUDE.md v1仕様） ===

/// 矢印キー集合A = {↑, ↓, ←, →}
pub const ARROW_KEYS: [KeyId; 4] = [
    KeyId::Arrow(ArrowKey::Up),
    KeyId::Arrow(ArrowKey::Down),
    KeyId::Arrow(ArrowKey::Left),
    KeyId::Arrow(ArrowKey::Right),
];

/// 数字キー集合N = {1,2,3,4,5,6,7,8,9,0}（CLAUDE.md順序）
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

// === v1モデル構造体（数理定義に対応） ===

/// 通常キー配置候補: x_{k,j,w}に対応
#[derive(Debug, Clone)]
pub struct KeyCandidate {
    pub key: KeyId,               // k ∈ K
    pub start_cell: CellId,       // j ∈ I_k^g
    pub width_u: f32,             // w ∈ W_k
    pub fitts_time_ms: f32,       // T^{(k)}_{j,w}（事前計算済み）
    pub cover_cells: Vec<CellId>, // C(j',j,w)の計算用
}

/// 矢印/数字用1uブロック
#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub center_u: (f32, f32),     // ブロック中心[u]
    pub cover_cells: [CellId; 4], // B(j',u)の計算用
}

// === v1メイン関数（CLAUDE.md v1仕様実装） ===

/// v1: 単一レイヤー・アルファベット固定モデル
pub fn solve_layout_v1(geom: &mut Geometry, freqs: &KeyFreq, config: &Config) -> Result<Solution> {
    log::info!("=== v1モデル開始: 単一レイヤー・アルファベット固定 ===");

    // 1. 集合の定義（CLAUDE.md Section 1）
    let movable_keys = extract_movable_keys(config);
    let key_candidates = generate_key_candidates(geom, &movable_keys)?;
    let blocks = generate_blocks(geom);
    let adjacency_edges = generate_adjacency(&blocks);

    log::info!(
        "キー候補: {}, ブロック: {}, エッジ: {}",
        key_candidates.len(),
        blocks.len(),
        adjacency_edges.len()
    );

    // 2. パラメータ設定（CLAUDE.md Section 7）
    let fingerwise_coeffs = FingerwiseFittsCoefficients::from_config(config);
    let probabilities = freqs.probabilities();

    // 3. 決定変数定義（CLAUDE.md Section 3）
    let mut vars = ProblemVariables::new();
    let (x_vars, a_vars, m_vars, r_vars, f_vars, d_vars, n_vars, dr_vars, df_vars) =
        create_decision_variables(
            &mut vars,
            &key_candidates,
            &blocks,
            &adjacency_edges,
            config,
        );

    // 4. 目的関数構築（CLAUDE.md Section 4）
    let objective = build_objective_function(
        &key_candidates,
        &blocks,
        &probabilities,
        &fingerwise_coeffs,
        &x_vars,
        &m_vars,
        &n_vars,
        config,
        geom,
    )?;

    // 5. 制約条件追加（CLAUDE.md Section 5）
    let mut model = vars.minimise(objective).using(coin_cbc);
    model = add_constraints(
        model,
        geom,
        &movable_keys,
        &key_candidates,
        &blocks,
        &adjacency_edges,
        &x_vars,
        &a_vars,
        &m_vars,
        &r_vars,
        &f_vars,
        &d_vars,
        &n_vars,
        &dr_vars,
        &df_vars,
        config,
    )?;

    // 6. 最適化実行
    log::info!("v1 MILPモデル求解開始");
    let solution = model
        .solve()
        .map_err(|e| crate::error::KbOptError::Solver(format!("v1最適化失敗: {}", e)))?;

    // 7. 結果構築
    let result = build_solution(SolutionBuildContext {
        solution: &solution,
        geom,
        key_candidates: &key_candidates,
        blocks: &blocks,
        x_vars: &x_vars,
        m_vars: &m_vars,
        n_vars: &n_vars,
        config,
    })?;

    log::info!("=== v1モデル完了: 目的値 {:.2}ms ===", result.objective_ms);
    Ok(result)
}

// === 集合とパラメータ生成（CLAUDE.md Section 1, 2） ===

/// 最適化対象キー集合K
fn extract_movable_keys(config: &Config) -> Vec<KeyId> {
    all_movable_keys(config)
        .into_iter()
        .filter(|k| !is_arrow(k) && !is_digit(k)) // 矢印・数字は別途クラスタ処理
        .collect()
}

/// キー配置候補生成: (j,w) ∈ I_k^g × W_k
fn generate_key_candidates(geom: &Geometry, movable_keys: &[KeyId]) -> Result<Vec<KeyCandidate>> {
    let mut candidates = Vec::new();
    let fingerwise_coeffs = FingerwiseFittsCoefficients::default(); // 一時的

    for &key in movable_keys {
        let widths = allowed_widths(&key);

        // 全行に配置可能（v1仕様）
        for row in 0..geom.cells.len() {
            for col in 0..geom.cells[row].len() {
                if geom.cells[row][col].occupied {
                    continue;
                }

                let start_cell = CellId::new(row, col);

                for &width_u in widths {
                    let width_cells = (width_u * U2CELL as f32) as usize;

                    // セル範囲チェック
                    if col + width_cells > geom.cells[row].len() {
                        continue;
                    }

                    // 占有セル計算
                    let cover_cells: Vec<CellId> = (col..col + width_cells)
                        .map(|c| CellId::new(row, c))
                        .collect();

                    // 全セルが空きかチェック
                    if cover_cells
                        .iter()
                        .all(|&cell| !geom.cells[cell.row][cell.col].occupied)
                    {
                        // Fitts時間計算（事前計算）
                        let key_center_mm = (
                            (col as f32 + width_u * U2CELL as f32 / 2.0) / U2CELL as f32 * U2MM,
                            (row as f32 + 0.5) * U2MM,
                        );
                        let finger = geom.cells[row][col].finger;
                        let home_pos = geom.homes.get(&finger).copied().unwrap_or(key_center_mm);

                        let fitts_time_ms = compute_fitts_time(
                            finger,
                            key_center_mm,
                            home_pos,
                            width_u,
                            &fingerwise_coeffs,
                        )?;

                        candidates.push(KeyCandidate {
                            key,
                            start_cell,
                            width_u,
                            fitts_time_ms,
                            cover_cells,
                        });
                    }
                }
            }
        }
    }

    Ok(candidates)
}

/// 1uブロック集合U_g生成（矢印・数字用）
fn generate_blocks(geom: &Geometry) -> Vec<Block> {
    let mut blocks = Vec::new();

    for row in 0..geom.cells.len() {
        let mut col = 0;
        while col + U2CELL <= geom.cells[row].len() {
            // 4セル全てが空きかチェック
            let cells: Vec<CellId> = (col..col + U2CELL).map(|c| CellId::new(row, c)).collect();

            if cells
                .iter()
                .all(|&cell| !geom.cells[cell.row][cell.col].occupied)
            {
                let block_id = BlockId::new(row, col / U2CELL);
                let center_u = (col as f32 / U2CELL as f32 + 0.5, row as f32 + 0.5);

                blocks.push(Block {
                    id: block_id,
                    center_u,
                    cover_cells: [cells[0], cells[1], cells[2], cells[3]],
                });
            }
            col += U2CELL;
        }
    }

    blocks
}

/// 近傍グラフE_g（4近傍）
fn generate_adjacency(blocks: &[Block]) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    let block_map: HashMap<BlockId, usize> = blocks
        .iter()
        .enumerate()
        .map(|(i, block)| (block.id, i))
        .collect();

    for (i, block) in blocks.iter().enumerate() {
        let (row, col) = (block.id.row_u, block.id.col_u);
        let neighbors = [
            BlockId::new(row, col + 1),
            BlockId::new(row, col.wrapping_sub(1)),
            BlockId::new(row + 1, col),
            BlockId::new(row.wrapping_sub(1), col),
        ];

        for neighbor_id in neighbors {
            if let Some(&j) = block_map.get(&neighbor_id) {
                edges.push((i, j));
            }
        }
    }

    edges
}

// === 決定変数定義（CLAUDE.md Section 3） ===

#[allow(clippy::type_complexity)]
fn create_decision_variables(
    vars: &mut ProblemVariables,
    key_candidates: &[KeyCandidate],
    blocks: &[Block],
    adjacency_edges: &[(usize, usize)],
    config: &Config,
) -> (
    Vec<Variable>, // x_{k,j,w}
    Vec<Variable>, // a^A_u
    Vec<Variable>, // m^A_{d,u}
    Vec<Variable>, // r^A_u
    Vec<Variable>, // f^A_{u→v}
    Vec<Variable>, // d^N_u（数字用）
    Vec<Variable>, // n^N_{d,u}（数字用）
    Vec<Variable>, // dr^N_u（数字用）
    Vec<Variable>, // df^N_{u→v}（数字用）
) {
    // x_{k,j,w} ∈ {0,1}
    let x_vars: Vec<Variable> = (0..key_candidates.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // 矢印クラスタ変数
    let a_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();
    let m_vars: Vec<Variable> = (0..(ARROW_KEYS.len() * blocks.len()))
        .map(|_| vars.add(variable().binary()))
        .collect();
    let r_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();
    let f_vars: Vec<Variable> = (0..adjacency_edges.len())
        .map(|_| vars.add(variable().min(0.0)))
        .collect();

    // 数字クラスタ変数（include_digits時のみ）
    let (d_vars, n_vars, dr_vars, df_vars) = if config.solver.include_digits {
        (
            (0..blocks.len())
                .map(|_| vars.add(variable().binary()))
                .collect(),
            (0..(DIGIT_KEYS.len() * blocks.len()))
                .map(|_| vars.add(variable().binary()))
                .collect(),
            (0..blocks.len())
                .map(|_| vars.add(variable().binary()))
                .collect(),
            (0..adjacency_edges.len())
                .map(|_| vars.add(variable().min(0.0)))
                .collect(),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };

    (
        x_vars, a_vars, m_vars, r_vars, f_vars, d_vars, n_vars, dr_vars, df_vars,
    )
}

// === 目的関数（CLAUDE.md Section 4） ===

#[allow(clippy::too_many_arguments)]
fn build_objective_function(
    key_candidates: &[KeyCandidate],
    blocks: &[Block],
    probabilities: &HashMap<KeyId, f64>,
    fingerwise_coeffs: &FingerwiseFittsCoefficients,
    x_vars: &[Variable],
    m_vars: &[Variable],
    n_vars: &[Variable],
    config: &Config,
    geom: &Geometry,
) -> Result<Expression> {
    let mut objective = Expression::from(0.0);

    // α Σ_{k∈K} Σ_{j,w} f_k T^{(k)}_{j,w} x_{k,j,w}
    for (i, candidate) in key_candidates.iter().enumerate() {
        let freq = probabilities.get(&candidate.key).copied().unwrap_or(0.0);
        let cost = freq * candidate.fitts_time_ms as f64;
        objective += cost * x_vars[i];
    }

    // β Σ_{d∈A} Σ_u f_d T_tap(u) m^A_{d,u}
    for (arrow_idx, &arrow_key) in ARROW_KEYS.iter().enumerate() {
        let freq = probabilities.get(&arrow_key).copied().unwrap_or(0.0);
        for (block_idx, block) in blocks.iter().enumerate() {
            let var_idx = arrow_idx * blocks.len() + block_idx;

            // 矢印のFitts時間計算
            let fitts_time_ms = compute_block_fitts_time(block, geom, fingerwise_coeffs)?;
            let cost = freq * fitts_time_ms as f64;
            objective += cost * m_vars[var_idx];
        }
    }

    // β Σ_{d∈N} Σ_u f_d T_tap(u) n^N_{d,u}（数字クラスタ有効時）
    if config.solver.include_digits && !n_vars.is_empty() {
        for (digit_idx, &digit_key) in DIGIT_KEYS.iter().enumerate() {
            let freq = probabilities.get(&digit_key).copied().unwrap_or(0.0);
            for (block_idx, block) in blocks.iter().enumerate() {
                let var_idx = digit_idx * blocks.len() + block_idx;

                let fitts_time_ms = compute_block_fitts_time(block, geom, fingerwise_coeffs)?;
                let cost = freq * fitts_time_ms as f64;
                objective += cost * n_vars[var_idx];
            }
        }
    }

    Ok(objective)
}

/// ブロックのFitts時間計算: T_tap(u)
fn compute_block_fitts_time(
    block: &Block,
    geom: &Geometry,
    coeffs: &FingerwiseFittsCoefficients,
) -> Result<f32> {
    let center_mm = (block.center_u.0 * U2MM, block.center_u.1 * U2MM);

    // ブロック中心の担当指を取得
    let center_row = block.center_u.1 as usize;
    let center_col = (block.center_u.0 * U2CELL as f32) as usize;
    let finger = geom.cells[center_row][center_col].finger;
    let home_pos = geom.homes.get(&finger).copied().unwrap_or(center_mm);

    compute_fitts_time(finger, center_mm, home_pos, 1.0, coeffs)
}

// === 制約条件（CLAUDE.md Section 5） ===

#[allow(clippy::too_many_arguments)]
fn add_constraints(
    mut model: good_lp::solvers::coin_cbc::CoinCbcProblem,
    geom: &Geometry,
    movable_keys: &[KeyId],
    key_candidates: &[KeyCandidate],
    blocks: &[Block],
    adjacency_edges: &[(usize, usize)],
    x_vars: &[Variable],
    a_vars: &[Variable],
    m_vars: &[Variable],
    r_vars: &[Variable],
    f_vars: &[Variable],
    d_vars: &[Variable],
    n_vars: &[Variable],
    dr_vars: &[Variable],
    df_vars: &[Variable],
    config: &Config,
) -> Result<good_lp::solvers::coin_cbc::CoinCbcProblem> {
    // 5.1) 一意性制約: Σ_{j,w} x_{k,j,w} = 1 ∀k∈K
    for &key in movable_keys {
        let key_indices: Vec<usize> = key_candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key == key)
            .map(|(i, _)| i)
            .collect();

        if !key_indices.is_empty() {
            let sum: Expression = key_indices.iter().map(|&i| x_vars[i]).sum();
            model = model.with(sum.eq(1.0));
        }
    }

    // 5.2) 物理非重複制約
    model =
        add_non_overlap_constraints(model, geom, key_candidates, blocks, x_vars, a_vars, d_vars)?;

    // 5.3) 矢印クラスタ制約
    model = add_arrow_cluster_constraints(
        model,
        blocks,
        adjacency_edges,
        a_vars,
        m_vars,
        r_vars,
        f_vars,
    )?;

    // 5.4) 数字クラスタ制約（有効時）
    if config.solver.include_digits && !d_vars.is_empty() {
        model = add_digit_cluster_constraints(
            model,
            blocks,
            adjacency_edges,
            d_vars,
            n_vars,
            dr_vars,
            df_vars,
        )?;
    }

    Ok(model)
}

/// 物理非重複制約: Σ C(j',j,w)·x_{k,j,w} + Σ B(j',u)·a_u + F^g_{j'} ≤ 1
fn add_non_overlap_constraints(
    mut model: good_lp::solvers::coin_cbc::CoinCbcProblem,
    geom: &Geometry,
    key_candidates: &[KeyCandidate],
    blocks: &[Block],
    x_vars: &[Variable],
    a_vars: &[Variable],
    d_vars: &[Variable],
) -> Result<good_lp::solvers::coin_cbc::CoinCbcProblem> {
    // セルごとに制約作成
    for row in 0..geom.cells.len() {
        for col in 0..geom.cells[row].len() {
            let cell_id = CellId::new(row, col);
            let mut constraint = Expression::from(if geom.cells[row][col].occupied {
                1.0
            } else {
                0.0
            });

            // 通常キーの占有
            for (i, candidate) in key_candidates.iter().enumerate() {
                if candidate.cover_cells.contains(&cell_id) {
                    constraint += x_vars[i];
                }
            }

            // 矢印ブロックの占有
            for (u, block) in blocks.iter().enumerate() {
                if block.cover_cells.contains(&cell_id) {
                    constraint += a_vars[u];
                }
            }

            // 数字ブロックの占有
            if !d_vars.is_empty() {
                for (u, block) in blocks.iter().enumerate() {
                    if block.cover_cells.contains(&cell_id) {
                        constraint += d_vars[u];
                    }
                }
            }

            model = model.with(constraint.leq(1.0));
        }
    }

    Ok(model)
}

/// 矢印クラスタ制約（CLAUDE.md Section 5.3）
fn add_arrow_cluster_constraints(
    mut model: good_lp::solvers::coin_cbc::CoinCbcProblem,
    blocks: &[Block],
    adjacency_edges: &[(usize, usize)],
    a_vars: &[Variable],
    m_vars: &[Variable],
    r_vars: &[Variable],
    f_vars: &[Variable],
) -> Result<good_lp::solvers::coin_cbc::CoinCbcProblem> {
    // Σ_u a^A_u = 4
    let arrow_count: Expression = a_vars.iter().cloned().sum();
    model = model.with(arrow_count.eq(4.0));

    // Σ_u m^A_{d,u} = 1 ∀d∈A
    for arrow_idx in 0..ARROW_KEYS.len() {
        let assignment_sum: Expression = (0..blocks.len())
            .map(|block_idx| m_vars[arrow_idx * blocks.len() + block_idx])
            .sum();
        model = model.with(assignment_sum.eq(1.0));
    }

    // m^A_{d,u} ≤ a^A_u
    for arrow_idx in 0..ARROW_KEYS.len() {
        #[allow(clippy::needless_range_loop)]
        for block_idx in 0..blocks.len() {
            let var_idx = arrow_idx * blocks.len() + block_idx;
            model = model.with((m_vars[var_idx] - a_vars[block_idx]).leq(0.0));
        }
    }

    // フロー保存則とルート制約
    model = add_flow_constraints(model, blocks, adjacency_edges, a_vars, r_vars, f_vars, 4.0)?;

    Ok(model)
}

/// 数字クラスタ制約（矢印と同構造、S_N=10）
fn add_digit_cluster_constraints(
    mut model: good_lp::solvers::coin_cbc::CoinCbcProblem,
    blocks: &[Block],
    adjacency_edges: &[(usize, usize)],
    d_vars: &[Variable],
    n_vars: &[Variable],
    dr_vars: &[Variable],
    df_vars: &[Variable],
) -> Result<good_lp::solvers::coin_cbc::CoinCbcProblem> {
    // Σ_u d^N_u = 10
    let digit_count: Expression = d_vars.iter().cloned().sum();
    model = model.with(digit_count.eq(10.0));

    // Σ_u n^N_{d,u} = 1 ∀d∈N
    for digit_idx in 0..DIGIT_KEYS.len() {
        let assignment_sum: Expression = (0..blocks.len())
            .map(|block_idx| n_vars[digit_idx * blocks.len() + block_idx])
            .sum();
        model = model.with(assignment_sum.eq(1.0));
    }

    // n^N_{d,u} ≤ d^N_u
    for digit_idx in 0..DIGIT_KEYS.len() {
        #[allow(clippy::needless_range_loop)]
        for block_idx in 0..blocks.len() {
            let var_idx = digit_idx * blocks.len() + block_idx;
            model = model.with((n_vars[var_idx] - d_vars[block_idx]).leq(0.0));
        }
    }

    // フロー保存則とルート制約（数字用）
    model = add_flow_constraints(
        model,
        blocks,
        adjacency_edges,
        d_vars,
        dr_vars,
        df_vars,
        10.0,
    )?;

    Ok(model)
}

/// フロー保存制約（矢印・数字共通）
fn add_flow_constraints(
    mut model: good_lp::solvers::coin_cbc::CoinCbcProblem,
    blocks: &[Block],
    adjacency_edges: &[(usize, usize)],
    usage_vars: &[Variable], // a_vars or d_vars
    root_vars: &[Variable],  // r_vars or dr_vars
    flow_vars: &[Variable],  // f_vars or df_vars
    cluster_size: f64,       // 4.0 or 10.0
) -> Result<good_lp::solvers::coin_cbc::CoinCbcProblem> {
    // フロー保存則: Σ_{v→u} f_{v→u} - Σ_{u→v} f_{u→v} = usage_u - cluster_size·root_u
    #[allow(clippy::needless_range_loop)]
    for block_idx in 0..blocks.len() {
        let mut flow_balance = Expression::from(0.0);

        // 流入
        for (edge_idx, &(_, target)) in adjacency_edges.iter().enumerate() {
            if target == block_idx {
                flow_balance += flow_vars[edge_idx];
            }
        }

        // 流出
        for (edge_idx, &(source, _)) in adjacency_edges.iter().enumerate() {
            if source == block_idx {
                flow_balance -= flow_vars[edge_idx];
            }
        }

        // バランス式
        let balance = usage_vars[block_idx] - cluster_size * root_vars[block_idx];
        model = model.with(flow_balance.eq(balance));
    }

    // ルート一意性: Σ_u root_u = 1
    let root_sum: Expression = root_vars.iter().cloned().sum();
    model = model.with(root_sum.eq(1.0));

    // フロー容量制約: f_{u→v} ≤ (cluster_size-1)·usage_u
    let max_flow = cluster_size - 1.0;
    for (edge_idx, &(source, _)) in adjacency_edges.iter().enumerate() {
        model = model.with((flow_vars[edge_idx] - max_flow * usage_vars[source]).leq(0.0));
    }

    Ok(model)
}

// === 解の構築 ===

struct SolutionBuildContext<'a> {
    solution: &'a dyn good_lp::Solution,
    geom: &'a mut Geometry,
    key_candidates: &'a [KeyCandidate],
    blocks: &'a [Block],
    x_vars: &'a [Variable],
    m_vars: &'a [Variable],
    n_vars: &'a [Variable],
    config: &'a Config,
}

fn build_solution(ctx: SolutionBuildContext) -> Result<Solution> {
    let mut total_cost = 0.0;
    let threshold = ctx.config.solver.solution_threshold;

    // 既存の最適化キーをクリア
    ctx.geom
        .key_placements
        .retain(|_, p| p.placement_type == PlacementType::Fixed);

    // 通常キー配置
    for (i, candidate) in ctx.key_candidates.iter().enumerate() {
        if ctx.solution.value(ctx.x_vars[i]) > threshold {
            let center_mm = (
                (candidate.start_cell.col as f32 + candidate.width_u * U2CELL as f32 / 2.0)
                    / U2CELL as f32
                    * U2MM,
                (candidate.start_cell.row as f32 + 0.5) * U2MM,
            );

            let placement = KeyPlacement {
                placement_type: PlacementType::Optimized,
                key_id: Some(candidate.key),
                x: center_mm.0,
                y: center_mm.1,
                width_u: candidate.width_u,
                block_id: None,
                layer: 0,
            };

            ctx.geom
                .key_placements
                .insert(format!("{:?}", candidate.key), placement);
            total_cost += candidate.fitts_time_ms as f64;
        }
    }

    // 矢印キー配置
    for (arrow_idx, &arrow_key) in ARROW_KEYS.iter().enumerate() {
        for (block_idx, block) in ctx.blocks.iter().enumerate() {
            let var_idx = arrow_idx * ctx.blocks.len() + block_idx;
            if ctx.solution.value(ctx.m_vars[var_idx]) > threshold {
                let center_mm = (block.center_u.0 * U2MM, block.center_u.1 * U2MM);

                let placement = KeyPlacement {
                    placement_type: PlacementType::Arrow,
                    key_id: Some(arrow_key),
                    x: center_mm.0,
                    y: center_mm.1,
                    width_u: 1.0,
                    block_id: Some(block.id),
                    layer: 0,
                };

                ctx.geom
                    .key_placements
                    .insert(format!("{:?}", arrow_key), placement);
            }
        }
    }

    // 数字キー配置
    if ctx.config.solver.include_digits && !ctx.n_vars.is_empty() {
        for (digit_idx, &digit_key) in DIGIT_KEYS.iter().enumerate() {
            for (block_idx, block) in ctx.blocks.iter().enumerate() {
                let var_idx = digit_idx * ctx.blocks.len() + block_idx;
                if ctx.solution.value(ctx.n_vars[var_idx]) > threshold {
                    let center_mm = (block.center_u.0 * U2MM, block.center_u.1 * U2MM);

                    let placement = KeyPlacement {
                        placement_type: PlacementType::Digit,
                        key_id: Some(digit_key),
                        x: center_mm.0,
                        y: center_mm.1,
                        width_u: 1.0,
                        block_id: Some(block.id),
                        layer: 0,
                    };

                    ctx.geom
                        .key_placements
                        .insert(format!("{:?}", digit_key), placement);
                }
            }
        }
    }

    Ok(Solution {
        objective_ms: total_cost,
    })
}

// === ユーティリティ関数 ===

fn is_arrow(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Arrow(_))
}

fn is_digit(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Digit(_))
}

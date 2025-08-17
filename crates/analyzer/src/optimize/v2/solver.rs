// v2ソルバー実装: 指別係数Fitts法則を使用した高度な最適化
// v2 solver implementation: Advanced optimization using finger-specific Fitts law coefficients
// Phase 0では v1と同じ実装を使用（将来の段階的実装に向けた準備）
// Phase 0 uses the same implementation as v1 (preparation for future phased implementation)

use super::SolveOptionsV2;
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;
use crate::geometry::Geometry;
use crate::optimize::{SolutionLayout, v1};

/// v2ソルバーのメインエントリポイント
/// Main entry point for v2 solver
///
/// Phase 0: v1互換実装 / v1-compatible implementation
/// Phase 1+: 指別Fitts係数、方向依存幅などの拡張機能を段階的に追加
/// Phase 1+: Progressive addition of advanced features like finger-specific Fitts coefficients, directional width
pub fn solve_layout_v2(
    geom: &mut Geometry,
    freqs: &KeyFreq,
    opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    // Phase 0: 基本チェックして、実装されていない機能に対してエラーを返す

    // v2統合ソルバー: 全Phase（1+2+3+4+5）を含む最終版
    log::info!("v2 solver: using integrated implementation (all phases)");
    solve_layout_integrated_v2(geom, freqs, opts)
}

/// v2統合ソルバー: 全Phase（1+2+3+4+5）を含む最終版
/// v2 integrated solver: Final version including all phases (1+2+3+4+5)
pub fn solve_layout_integrated_v2(
    geom: &mut Geometry,
    freqs: &KeyFreq,
    opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    use super::fitts::FittsCoefficients;
    use crate::geometry::types::{CellId, PlacementType};
    use crate::keys::{KeyId, ParseOptions, all_movable_keys};
    use good_lp::{
        Expression, ProblemVariables, Solution, SolverModel, Variable, coin_cbc, variable,
    };
    use std::collections::{BTreeSet, HashMap};

    // Phase 1: 指別Fitts係数を設定から作成
    let fitts_coeffs = if let Some(ref config) = opts.fitts_coeffs {
        FittsCoefficients::from_config(config)
    } else {
        FittsCoefficients::default()
    };

    log::info!("v2 integrated solver: Phase 1+2 implemented, Phase 3+4+5 preparation");

    /// 最適化モデルの定数
    const REQUIRED_ARROW_BLOCKS: usize = 4;
    const REQUIRED_ARROW_BLOCKS_F64: f64 = 4.0;
    const FLOW_ROOTS_F64: f64 = 1.0;
    const MAX_FLOW_PER_BLOCK: f64 = 3.0;

    // 1) 集合を作る - CSVにないキーも含める（count 0として扱う）
    let parse_opt = ParseOptions {
        include_fkeys: opts.base.include_fkeys,
        ..Default::default()
    };

    let movable: BTreeSet<KeyId> = all_movable_keys(&parse_opt)
        .into_iter()
        .filter(|k| !v1::solver::is_arrow(k))
        .collect();

    // 2) 最適化セットを生成
    let movable_vec: Vec<KeyId> = movable.iter().cloned().collect();
    let key_cands = v1::solver::generate_v1_key_candidates(geom, &movable_vec);
    let (arrow_cells, arrow_edges) = v1::solver::generate_v1_arrow_region(geom);

    let optimization_sets = crate::geometry::sets::OptimizationSets {
        key_cands,
        arrow_cells,
        arrow_edges,
    };

    // 3) Phase 1+2: 通常キーの候補を生成（指別+方向依存Fitts係数付き）
    let cands =
        build_directional_aware_candidates(geom, &movable, &optimization_sets, &fitts_coeffs);
    if cands.is_empty() {
        return Err(crate::error::KbOptError::ModelError {
            message: "No valid key placement candidates found".to_string(),
        });
    }

    // 4) 矢印用ブロックを生成
    let (blocks, _block_index) = v1::solver::build_blocks_from_precompute(geom, &optimization_sets);
    if blocks.len() < REQUIRED_ARROW_BLOCKS {
        return Err(crate::error::KbOptError::ModelError {
            message: format!(
                "Insufficient arrow blocks: found {}, need at least {}",
                blocks.len(),
                REQUIRED_ARROW_BLOCKS
            ),
        });
    }

    let adj_edges = v1::solver::build_adjacency_from_precompute(&blocks, &optimization_sets);

    // 5) モデルを立てる
    let mut vars = ProblemVariables::new();

    // x^g_{k,j,w}（二値）：通常キー配置変数
    let x_vars: Vec<Variable> = (0..cands.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // a^g_u（二値）：ブロック占有変数
    let a_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // m^g_{a,u}：矢印キー配置変数（4種×ブロック）
    let mut m_vars: HashMap<(KeyId, usize), Variable> = HashMap::new();
    for &arrow_key in &v1::solver::ARROW_KEYS {
        for u in 0..blocks.len() {
            m_vars.insert((arrow_key, u), vars.add(variable().binary()));
        }
    }

    // r^g_u（二値）：フロー根変数
    let r_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // f^g_{(u→v)}（連続）：フロー変数
    #[derive(Clone)]
    struct Edge {
        from: usize,
        to: usize,
    }
    let edges: Vec<Edge> = adj_edges
        .iter()
        .map(|(u, v)| Edge { from: *u, to: *v })
        .collect();
    let f_vars: Vec<Variable> = (0..edges.len())
        .map(|_| vars.add(variable().min(0.0)))
        .collect();

    // 正規化された確率値を取得
    let probabilities = freqs.probabilities();

    // Phase 3: TODO レイヤシステム変数の追加
    // - モディファイア配置変数 q_{l,m}
    // - レイヤ記号配置変数 z_{s,l,u,m}

    // Phase 4: TODO 数値クラスター変数の追加
    // - 数値開始位置変数 s_{r,b}
    // - 数値使用変数 a^{num}_u
    // - 数値割当変数 m^{num}_{d,u}

    // Phase 5: TODO ビグラム近似変数の追加
    // - 連動変数 y_{i,j} (TopM linearization approach)

    // 目的関数: v2統合（Phase 1+2実装済み、Phase 3+4+5拡張ポイント）
    let mut obj = Expression::from(0.0);

    // Phase 1+2: 通常キー項（指別+方向依存係数でFitts時間を計算済み）
    for (i, cand) in cands.iter().enumerate() {
        let p_k = probabilities.get(&cand.key).copied().unwrap_or(0.0);
        let effective_p_k = if p_k > 0.0 { p_k } else { 1e-6 };
        obj += effective_p_k * cand.cost_ms * x_vars[i];
    }

    // Phase 1+2: 矢印キー項（指別+方向依存係数）
    for (u, blk) in blocks.iter().enumerate() {
        let center_cell = blk.cover_cells[2]; // 中央近傍
        let finger = geom.cells[center_cell.row][center_cell.col].finger;
        let home = geom.homes.get(&finger).cloned().unwrap_or((
            blk.center.0 * crate::constants::U2MM as f32,
            blk.center.1 * crate::constants::U2MM as f32,
        ));

        let center_mm = (
            blk.center.0 * crate::constants::U2MM as f32,
            blk.center.1 * crate::constants::U2MM as f32,
        );
        let center_home_dist = crate::constants::euclid_distance(center_mm, home) as f64;
        let direction_phi = super::fitts::compute_direction_angle(home, center_mm);
        let t_ms = super::fitts::compute_fitts_time_directional(
            finger,
            center_home_dist,
            1.0, // 矢印キーは1u固定
            direction_phi,
            &fitts_coeffs,
        );

        for &arrow_key in &v1::solver::ARROW_KEYS {
            let p_a = probabilities.get(&arrow_key).copied().unwrap_or(0.0);
            let effective_p_a = if p_a > 0.0 { p_a } else { 1e-6 };
            let m_au = m_vars.get(&(arrow_key, u)).unwrap();
            obj += (effective_p_a * t_ms) * *m_au;
        }
    }

    // Phase 3: TODO レイヤシステム目的関数項の追加
    // obj += sum_{s,l,u,m} f_s * T^{chord}(u,m) * z_{s,l,u,m}

    // Phase 4: TODO 数値クラスター目的関数項の追加
    // obj += sum_{d,u} f_d * T_tap(u) * m^{num}_{d,u}

    // Phase 5: TODO ビグラム近似目的関数項の追加
    // obj += sum_{i,j} f_{i->j} * T_{i->j} * y_{i,j}

    // 目的関数を後で評価するために保存
    let objective_expr = obj.clone();

    // 6) 制約条件（v2統合版: Phase 1+2実装済み、Phase 3+4+5拡張ポイント）
    let mut model = vars.minimise(obj).using(coin_cbc);

    // (i) 一意性制約
    for &key in movable.iter() {
        let idxs: Vec<usize> = cands
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key == key)
            .map(|(i, _)| i)
            .collect();
        if !idxs.is_empty() {
            let sum: Expression = idxs.iter().map(|i| x_vars[*i]).sum();
            model = model.with(sum.clone().eq(1));
        }
    }

    // (ii) セル非重複制約
    let mut cell_cover_x: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (i, cand) in cands.iter().enumerate() {
        for cid in &cand.cover_cells {
            cell_cover_x.entry(*cid).or_default().push(i);
        }
    }
    let mut cell_cover_a: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (u, blk) in blocks.iter().enumerate() {
        for cid in &blk.cover_cells {
            cell_cover_a.entry(*cid).or_default().push(u);
        }
    }
    for r in 0..geom.cells.len() {
        for c in 0..geom.cells[r].len() {
            let cid = CellId::new(r, c);
            let fixed = if geom.cells[r][c].occupied { 1.0 } else { 0.0 };
            let xs = cell_cover_x.get(&cid).cloned().unwrap_or_default();
            let as_ = cell_cover_a.get(&cid).cloned().unwrap_or_default();
            let mut sum = Expression::from(fixed);
            for i in xs {
                sum += x_vars[i];
            }
            for u in as_ {
                sum += a_vars[u];
            }
            model = model.with(sum << 1);
        }
    }

    // (iii) 矢印キー制約（v1と同じ）
    for &arrow_key in &v1::solver::ARROW_KEYS {
        let sum: Expression = (0..blocks.len())
            .map(|u| *m_vars.get(&(arrow_key, u)).unwrap())
            .sum();
        model = model.with(sum.eq(1));
    }
    for (u, _) in a_vars.iter().enumerate().take(blocks.len()) {
        let sum_a: Expression = v1::solver::ARROW_KEYS
            .iter()
            .map(|&arrow_key| *m_vars.get(&(arrow_key, u)).unwrap())
            .sum();
        model = model.with(sum_a << a_vars[u]);
    }
    let sum_a_total: Expression = (0..blocks.len()).map(|u| a_vars[u]).sum();
    model = model.with(sum_a_total.eq(REQUIRED_ARROW_BLOCKS_F64));

    // (iv) フロー制約（v1と同じ）
    let sum_r: Expression = (0..blocks.len()).map(|u| r_vars[u]).sum();
    model = model.with(sum_r.eq(FLOW_ROOTS_F64));

    let mut in_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    for (e_idx, e) in edges.iter().enumerate() {
        out_edges[e.from].push(e_idx);
        in_edges[e.to].push(e_idx);
    }
    for u in 0..blocks.len() {
        let sum_in: Expression = in_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        let sum_out: Expression = out_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        model =
            model.with((sum_in - sum_out).eq(a_vars[u] - REQUIRED_ARROW_BLOCKS_F64 * r_vars[u]));
    }
    for (e_idx, e) in edges.iter().enumerate() {
        model = model.with(f_vars[e_idx] << (MAX_FLOW_PER_BLOCK * a_vars[e.from]));
    }

    // Phase 3: TODO レイヤシステム制約の追加
    // - モディファイア一意性: sum_m q_{l,m} <= 1
    // - レイヤ配置整合性: z_{s,l,u,m} <= q_{l,m}
    // - 記号配置一意性: sum_j x_{s,j,w} + sum_{l,u,m} z_{s,l,u,m} = 1

    // Phase 4: TODO 数値クラスター制約の追加
    // - 開始位置一意性: sum_{r,b} s_{r,b} = 1
    // - 連続配置: a^{num}_{u(r,b+t-1)} = s_{r,b}
    // - 数字順序固定: m^{num}_{d,u(r,b+t(d)-1)} = s_{r,b}

    // Phase 5: TODO ビグラム近似制約の追加
    // - 連動制約: y_{i,j} <= x_i, y_{i,j} <= x_j, y_{i,j} >= x_i + x_j - 1

    // 7) 求解
    let sol = model.solve().map_err(|e| {
        crate::error::KbOptError::SolverError(format!("Failed to solve optimization model: {}", e))
    })?;

    // 8) 解の復元
    let objective_ms = sol.eval(&objective_expr);

    // 既存の最適化キーをクリア（固定キーは残す）
    geom.key_placements
        .retain(|_, p| p.placement_type == PlacementType::Fixed);

    // 通常キーの配置を追加
    let _regular_keys_placed = apply_regular_key_placements_v2(geom, &sol, &x_vars, &cands);

    // 矢印キーの配置を追加
    let _arrow_keys_placed = apply_arrow_key_placements_v2(geom, &sol, &m_vars, &blocks);

    Ok(crate::optimize::SolutionLayout { objective_ms })
}

/// v2統合版: 指別+方向依存Fitts係数を考慮したキー候補を生成（Phase 1+2実装済み）
fn build_directional_aware_candidates(
    geom: &Geometry,
    movable: &std::collections::BTreeSet<crate::keys::KeyId>,
    optimization_sets: &crate::geometry::sets::OptimizationSets,
    fitts_coeffs: &super::fitts::FittsCoefficients,
) -> Vec<v1::solver::Cand> {
    use crate::constants::U2CELL;

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

                    // Phase 1+2: 指別+方向依存係数でFitts時間を計算
                    let key_center_mm = (
                        cx * crate::constants::U2MM as f32,
                        cy * crate::constants::U2MM as f32,
                    );
                    let distance_mm = crate::constants::euclid_distance(key_center_mm, home) as f64;
                    let direction_phi = super::fitts::compute_direction_angle(home, key_center_mm);
                    let t_ms = super::fitts::compute_fitts_time_directional(
                        finger,
                        distance_mm,
                        w_u,
                        direction_phi,
                        fitts_coeffs,
                    );

                    // 新しい座標系: 行はu単位、列はcell単位
                    let cover_cells: Vec<crate::geometry::types::CellId> = (start_cell.col
                        ..start_cell.col + w_cells)
                        .map(|cc| crate::geometry::types::CellId::new(start_cell.row, cc))
                        .collect();

                    out.push(v1::solver::Cand {
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

/// Phase 1: 通常キーの配置を適用（v2用）
fn apply_regular_key_placements_v2(
    geom: &mut crate::geometry::Geometry,
    sol: &dyn good_lp::Solution,
    x_vars: &[good_lp::Variable],
    cands: &[v1::solver::Cand],
) -> usize {
    use crate::geometry::types::{KeyPlacement, PlacementType};

    const SOLUTION_THRESHOLD: f64 = 0.5;
    let mut placed = 0;

    for (i, cand) in cands.iter().enumerate() {
        let var_value = sol.value(x_vars[i]);

        if var_value > SOLUTION_THRESHOLD {
            let (x, y) = crate::constants::cell_to_key_center(cand.row, cand.start_col, cand.w_u);

            geom.key_placements.insert(
                cand.key.to_string(),
                KeyPlacement {
                    placement_type: PlacementType::Optimized,
                    key_id: Some(cand.key),
                    x,
                    y,
                    width_u: cand.w_u,
                    block_id: None,
                },
            );
            placed += 1;
        }
    }

    placed
}

/// Phase 1: 矢印キーの配置を適用（v2用）
fn apply_arrow_key_placements_v2(
    geom: &mut crate::geometry::Geometry,
    sol: &dyn good_lp::Solution,
    m_vars: &std::collections::HashMap<(crate::keys::KeyId, usize), good_lp::Variable>,
    blocks: &[v1::solver::Block],
) -> usize {
    use crate::geometry::types::{KeyPlacement, PlacementType};

    const SOLUTION_THRESHOLD: f64 = 0.5;
    let mut placed = 0;

    for &arrow_key in &v1::solver::ARROW_KEYS {
        for (u, block) in blocks.iter().enumerate() {
            let var_value = sol.value(*m_vars.get(&(arrow_key, u)).unwrap());

            if var_value > SOLUTION_THRESHOLD {
                let start_col = block.id.col_u * crate::constants::U2CELL; // 1u = 4 cells
                let (x, y) = crate::constants::cell_to_key_center(block.id.row_u, start_col, 1.0);

                geom.key_placements.insert(
                    arrow_key.to_string(),
                    KeyPlacement {
                        placement_type: PlacementType::Arrow,
                        key_id: Some(arrow_key),
                        x,
                        y,
                        width_u: 1.0,
                        block_id: Some(block.id),
                    },
                );
                placed += 1;
            }
        }
    }
    placed
}

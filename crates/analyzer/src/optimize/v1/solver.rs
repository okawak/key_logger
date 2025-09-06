use crate::{
    config::Config,
    constants::{U2CELL, U2MM},
    csv_reader::KeyFreq,
    error::{KbOptError, Result},
    geometry::{
        Geometry,
        types::{KeyPlacement, PlacementType},
    },
    keys::KeyId,
    optimize::{
        Solution,
        fitts::{FingerwiseFittsCoefficients, compute_fitts_time},
        precompute::{PrecomputedFitts, all_movable_keys, precompute_fitts_times},
        v1::arrows::{ArrowPlacement, generate_horizontal_candidates, generate_t_shape_candidates},
    },
};
use good_lp::{
    Expression, ProblemVariables, SolverModel, Variable, highs, solvers::highs::HighsProblem,
    variable,
};
use std::collections::HashMap;

/// docs/v1.mdに従った最適化
pub fn solve_layout_v1(geom: &mut Geometry, freqs: &KeyFreq, config: &Config) -> Result<Solution> {
    log::info!("=== start v1 model optimization ===");

    // 1. 指別Fitts係数の準備
    let fingerwise_coeffs = FingerwiseFittsCoefficients::from_config(config);
    log::debug!("fitts coefficient: {:#?}", &fingerwise_coeffs);

    // 2. Fitts時間の事前計算（MILP化）
    let precomputed = precompute_fitts_times(geom, &fingerwise_coeffs)?;
    log::info!(
        "complete fitts time caluclation: {} candidates",
        precomputed.candidates.len()
    );

    // 3. 最適化対象キー集合Kの抽出
    let movable_keys = all_movable_keys(config);
    log::info!("key numbers: {}", movable_keys.len());

    // 4. 矢印キー配置候補の生成
    let horizontal_candidates = generate_horizontal_candidates(geom);
    let t_shape_candidates = generate_t_shape_candidates(geom);
    log::info!(
        "arrow keys candidate: horizontal={}, T shape={}",
        horizontal_candidates.len(),
        t_shape_candidates.len()
    );

    // 5. 決定変数の定義
    let mut vars = ProblemVariables::new();
    let (x_vars, x_var_info, z_h_vars, z_t_vars) = create_decision_variables(
        &mut vars,
        &movable_keys,
        &precomputed,
        &horizontal_candidates,
        &t_shape_candidates,
    );

    // 6. 目的関数の構築
    let probabilities = freqs.probabilities();
    let objective = build_objective_function(
        &x_var_info,
        &horizontal_candidates,
        &t_shape_candidates,
        &probabilities,
        &precomputed,
        &x_vars,
        &z_h_vars,
        &z_t_vars,
    )?;

    // 7. 制約条件の追加
    let model = vars.minimise(objective).using(highs);
    let mut model = model
        .set_option("output_flag", true)
        .set_option("log_to_console", true)
        .set_option("log_file", "optimization.log");

    model = add_constraints(
        model,
        geom,
        &movable_keys,
        &x_var_info,
        &horizontal_candidates,
        &t_shape_candidates,
        &x_vars,
        &z_h_vars,
        &z_t_vars,
    )?;

    // 8. 最適化実行
    log::info!("start solving v1 model...");
    let solution = model
        .solve()
        .map_err(|e| KbOptError::Solver(format!("failed to solve: {}", e)))?;

    // 9. 解の構築
    let result = build_solution(
        &solution,
        geom,
        &x_var_info,
        &horizontal_candidates,
        &t_shape_candidates,
        &precomputed,
        &probabilities,
        &x_vars,
        &z_h_vars,
        &z_t_vars,
        config,
    )?;

    log::info!(
        "=== sucessfully completed!: objective {:.2}ms ===",
        result.objective_ms
    );
    Ok(result)
}

/// 決定変数の定義
///
/// 決定変数:
/// - x_{k,r,i,s} ∈ {0,1}: 通常キー配置変数 (k ∈ K, (r,i,s) ∈ C)
/// - z^H_{r,i} ∈ {0,1}: 横一列配置変数 ((r,i) ∈ Ω_H)
/// - z^T_{r,i} ∈ {0,1}: T字型配置変数 ((r,i) ∈ Ω_T)
#[allow(clippy::type_complexity)]
fn create_decision_variables(
    vars: &mut ProblemVariables,
    movable_keys: &[KeyId],
    precomputed: &PrecomputedFitts,
    horizontal_candidates: &[ArrowPlacement],
    t_shape_candidates: &[ArrowPlacement],
) -> (
    Vec<Variable>,                          // x_vars
    Vec<(KeyId, usize, usize, usize, f64)>, // x_var_info (キーと変数を対応づけるために必要)
    Vec<Variable>,                          // z_h_vars
    Vec<Variable>,                          // z_t_vars
) {
    let mut x_vars = Vec::new();
    let mut x_var_info = Vec::new();

    // x_{k,r,i,s} ∈ {0,1}
    for &key in movable_keys {
        for (&(r, i, s), &t) in precomputed.candidates.iter() {
            x_vars.push(vars.add(variable().binary()));
            x_var_info.push((key, r, i, s, t));
        }
    }

    // 矢印キー配置変数
    let z_h_vars: Vec<Variable> = (0..horizontal_candidates.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    let z_t_vars: Vec<Variable> = (0..t_shape_candidates.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    log::info!(
        "number of R^2: x_vars={}, z_h_vars={}, z_t_vars={}",
        x_vars.len(),
        z_h_vars.len(),
        z_t_vars.len()
    );

    (x_vars, x_var_info, z_h_vars, z_t_vars)
}

/// 目的関数の構築
///
/// min (通常キー時間 + 矢印キー時間)
/// = Σ_{k∈K} Σ_{(r,i,s)∈C} p_k T(r,i,s) x_{k,r,i,s}
///   + [横一列項 + T字型項]
#[allow(clippy::too_many_arguments)]
fn build_objective_function(
    x_var_info: &[(KeyId, usize, usize, usize, f64)],
    horizontal_candidates: &[ArrowPlacement],
    t_shape_candidates: &[ArrowPlacement],
    probabilities: &HashMap<KeyId, f64>,
    precomputed: &PrecomputedFitts,
    x_vars: &[Variable],
    z_h_vars: &[Variable],
    z_t_vars: &[Variable],
) -> Result<Expression> {
    let mut objective = Expression::from(0.0);

    // 通常キー時間項
    // Σ_{k∈K} Σ_{(r,i,s)∈C} p_k T(r,i,s) x_{k,r,i,s}
    for (i, &(key, _r, _i, _s, fitts_time)) in x_var_info.iter().enumerate() {
        let freq = probabilities.get(&key).copied().unwrap_or(0.0);
        let cost = freq * fitts_time;
        objective += cost * x_vars[i];
    }

    // 矢印キー時間項

    // 横一列配置の場合: Σ_{(r,i)∈Ω_H} [Σ_{m=0}^3 p_{a_m} T(r, i + m s₀, s₀)] z^H_{r,i}
    for (idx, placement) in horizontal_candidates.iter().enumerate() {
        let mut placement_cost: f64 = 0.0;

        for (arrow_key, r, i) in placement.get_arrow_positions() {
            let key_id = KeyId::Arrow(arrow_key);
            let freq = probabilities.get(&key_id).copied().unwrap_or(0.0);

            // T(r, i, s0=1u)
            if let Some(&fitts_time) = precomputed.candidates.get(&(r, i, U2CELL)) {
                placement_cost += freq * fitts_time;
            }
        }

        objective += placement_cost * z_h_vars[idx];
    }

    // T字型配置の場合
    for (idx, placement) in t_shape_candidates.iter().enumerate() {
        let mut placement_cost = 0.0;

        for (arrow_key, r, i) in placement.get_arrow_positions() {
            let key_id = KeyId::Arrow(arrow_key);
            let freq = probabilities.get(&key_id).copied().unwrap_or(0.0);

            // T(r, i, s0=1u)
            if let Some(&fitts_time) = precomputed.candidates.get(&(r, i, U2CELL)) {
                placement_cost += freq * fitts_time;
            }
        }

        objective += placement_cost * z_t_vars[idx];
    }

    Ok(objective)
}

/// 制約条件の追加
#[allow(clippy::too_many_arguments)]
fn add_constraints(
    mut model: HighsProblem,
    geom: &Geometry,
    movable_keys: &[KeyId],
    x_var_info: &[(KeyId, usize, usize, usize, f64)],
    horizontal_candidates: &[ArrowPlacement],
    t_shape_candidates: &[ArrowPlacement],
    x_vars: &[Variable],
    z_h_vars: &[Variable],
    z_t_vars: &[Variable],
) -> Result<HighsProblem> {
    // 一意性制約
    // Σ_{(r,i,s)∈C} x_{k,r,i,s} = 1 ∀k ∈ K
    for &key in movable_keys {
        let key_indices: Vec<usize> = x_var_info
            .iter()
            .enumerate()
            .filter(|(_, (k, _, _, _, _))| *k == key)
            .map(|(i, _)| i)
            .collect();

        if !key_indices.is_empty() {
            let sum: Expression = key_indices.iter().map(|&i| x_vars[i]).sum();
            model = model.with(sum.eq(1.0));
        }
    }

    // 矢印配置の一意性
    // Σ_{(r,i)∈Ω_H} z^H_{r,i} + Σ_{(r,i)∈Ω_T} z^T_{r,i} = 1
    let arrow_sum: Expression = z_h_vars
        .iter()
        .cloned()
        .chain(z_t_vars.iter().cloned())
        .sum();
    model = model.with(arrow_sum.eq(1.0));

    // 物理的非重複制約
    model = add_non_overlap_constraints(
        model,
        geom,
        x_var_info,
        horizontal_candidates,
        t_shape_candidates,
        x_vars,
        z_h_vars,
        z_t_vars,
    )?;

    Ok(model)
}

/// 物理的非重複制約
/// 各セルは最大1つのキーのみが占有可能
/// Σ_{k∈K} Σ_{(r,i,s)∈C, i≤j≤i+s-1} x_{k,r,i,s} + φ^arrow_{rj} + O_{rj} ≤ 1
#[allow(clippy::too_many_arguments)]
fn add_non_overlap_constraints(
    mut model: HighsProblem,
    geom: &Geometry,
    x_var_info: &[(KeyId, usize, usize, usize, f64)],
    horizontal_candidates: &[ArrowPlacement],
    t_shape_candidates: &[ArrowPlacement],
    x_vars: &[Variable],
    z_h_vars: &[Variable],
    z_t_vars: &[Variable],
) -> Result<HighsProblem> {
    // 各セルに対して制約を作成
    for r in 0..geom.cells.len() {
        for j in 0..geom.cells[r].len() {
            let mut constraint = Expression::from(if geom.cells[r][j].occupied {
                1.0 // O_{rj} = 1（既に占有）
            } else {
                0.0 // O_{rj} = 0（空き）
            });

            // 通常キーの占有: Σ_{k∈K} Σ_{(r,i,s)∈C, i≤j≤i+s-1} x_{k,r,i,s}
            for (idx, &(_key, row, i, s, _fitts_time)) in x_var_info.iter().enumerate() {
                if row == r && i <= j && j < i + s {
                    constraint += x_vars[idx];
                }
            }

            // 矢印占有関数 φ^arrow_{rj}

            // 横一列配置の占有
            for (idx, placement) in horizontal_candidates.iter().enumerate() {
                let occupied_cells = placement.get_occupied_cells();
                if occupied_cells.contains(&(r, j)) {
                    constraint += z_h_vars[idx];
                }
            }

            // T字型配置の占有
            for (idx, placement) in t_shape_candidates.iter().enumerate() {
                let occupied_cells = placement.get_occupied_cells();
                if occupied_cells.contains(&(r, j)) {
                    constraint += z_t_vars[idx];
                }
            }

            model = model.with(constraint.leq(1.0));
        }
    }

    Ok(model)
}

/// 固定キーの目的関数への寄与分を計算
fn calculate_fixed_keys_contribution(
    geom: &Geometry,
    probabilities: &HashMap<KeyId, f64>,
    fingerwise_coeffs: &FingerwiseFittsCoefficients,
) -> Result<f64> {
    let mut fixed_contribution = 0.0;

    for placement in geom.key_placements.values() {
        if placement.placement_type == PlacementType::Fixed
            && let Some(key_id) = placement.key_id
        {
            // 確率を取得
            let prob = probabilities.get(&key_id).copied().unwrap_or(0.0);
            if prob == 0.0 {
                continue;
            }

            // キー中心座標
            let key_center_mm = (placement.x, placement.y);

            // 座標からセル位置を逆算して担当指を取得
            let row = ((placement.y / U2MM).round() as usize).saturating_sub(1);
            let col =
                ((placement.x / U2MM * U2CELL as f64).round() as usize).saturating_sub(U2CELL / 2);

            if row >= geom.cells.len() || col >= geom.cells[row].len() {
                continue;
            }

            let finger = geom.cells[row][col].finger;

            // ホーム位置を取得
            let home_position = geom.homes.get(&finger).ok_or_else(|| {
                KbOptError::Config(format!("Home position not found for finger {:?}", finger))
            })?;

            // Fitts時間を計算
            let key_width_cell = (placement.width_u * U2CELL as f64) as usize;
            let fitts_time = compute_fitts_time(
                finger,
                key_center_mm,
                *home_position,
                key_width_cell,
                fingerwise_coeffs,
            )?;

            fixed_contribution += prob * fitts_time;
            log::debug!(
                "Fixed key {:?}: prob={:.6}, fitts_time={:.2}ms, contribution={:.4}ms",
                key_id,
                prob,
                fitts_time,
                prob * fitts_time
            );
        }
    }

    log::info!("Fixed keys total contribution: {:.2}ms", fixed_contribution);
    Ok(fixed_contribution)
}

/// 解の構築
#[allow(clippy::too_many_arguments)]
fn build_solution(
    solution: &dyn good_lp::Solution,
    geom: &mut Geometry,
    x_var_info: &[(KeyId, usize, usize, usize, f64)],
    horizontal_candidates: &[ArrowPlacement],
    t_shape_candidates: &[ArrowPlacement],
    precomputed: &PrecomputedFitts,
    probabilities: &HashMap<KeyId, f64>,
    x_vars: &[Variable],
    z_h_vars: &[Variable],
    z_t_vars: &[Variable],
    config: &Config,
) -> Result<Solution> {
    let threshold = config.solver.solution_threshold;
    let mut objective_value = 0.0;

    // 既存の最適化キーをクリア
    geom.key_placements
        .retain(|_, p| p.placement_type == PlacementType::Fixed);

    // 通常キー配置の復元と目的関数値計算
    for (idx, &(key, r, i, s, fitts_time)) in x_var_info.iter().enumerate() {
        if solution.value(x_vars[idx]) > threshold {
            let width_u = s as f64 / U2CELL as f64; // セル → u 変換

            // 中心座標計算
            let center_mm = (
                (i as f64 + s as f64 / 2.0) * (U2MM / U2CELL as f64), // Δ_mm = d_mm/4
                (r as f64 + 0.5) * U2MM,                              // y_r
            );

            let placement = KeyPlacement {
                placement_type: PlacementType::Optimized,
                key_id: Some(key),
                x: center_mm.0,
                y: center_mm.1,
                width_u,
                layer: 0,
            };

            geom.key_placements.insert(format!("{:?}", key), placement);

            // 目的関数値に貢献度を加算: p_k × T(r,i,s)
            let prob = probabilities.get(&key).copied().unwrap_or(0.0);
            objective_value += prob * fitts_time;
        }
    }

    // 矢印キー配置の復元と目的関数値計算
    for (idx, placement) in horizontal_candidates.iter().enumerate() {
        if solution.value(z_h_vars[idx]) > threshold {
            add_arrow_placements(geom, placement, precomputed);

            // 矢印キーの目的関数値貢献度を加算
            for (arrow_key, r, col) in placement.get_arrow_positions() {
                let key_id = KeyId::Arrow(arrow_key);
                let prob = probabilities.get(&key_id).copied().unwrap_or(0.0);
                if let Some(&fitts_time) = precomputed.candidates.get(&(r, col, U2CELL)) {
                    objective_value += prob * fitts_time;
                }
            }
        }
    }

    for (idx, placement) in t_shape_candidates.iter().enumerate() {
        if solution.value(z_t_vars[idx]) > threshold {
            add_arrow_placements(geom, placement, precomputed);

            // 矢印キーの目的関数値貢献度を加算
            for (arrow_key, r, col) in placement.get_arrow_positions() {
                let key_id = KeyId::Arrow(arrow_key);
                let prob = probabilities.get(&key_id).copied().unwrap_or(0.0);
                if let Some(&fitts_time) = precomputed.candidates.get(&(r, col, U2CELL)) {
                    objective_value += prob * fitts_time;
                }
            }
        }
    }

    // 固定キーの寄与分を計算して加算
    let fingerwise_coeffs = FingerwiseFittsCoefficients::from_config(config);
    let fixed_contribution =
        calculate_fixed_keys_contribution(geom, probabilities, &fingerwise_coeffs)?;

    let total_objective = objective_value + fixed_contribution;

    log::info!("Optimized keys contribution: {:.2}ms", objective_value);
    log::info!("Fixed keys contribution: {:.2}ms", fixed_contribution);
    log::info!("Total objective value: {:.2}ms", total_objective);

    Ok(Solution {
        objective_ms: total_objective,
    })
}

/// 矢印キー配置を geometry に追加
fn add_arrow_placements(
    geom: &mut Geometry,
    placement: &ArrowPlacement,
    _precomputed: &PrecomputedFitts,
) {
    for (arrow_key, r, col) in placement.get_arrow_positions() {
        let key_id = KeyId::Arrow(arrow_key);

        let center_mm = (
            (col as f64 / U2CELL as f64 + 0.5) * U2MM, // ブロック中心 x
            (r as f64 + 0.5) * U2MM,                   // ブロック中心 y
        );

        let placement = KeyPlacement {
            placement_type: PlacementType::Arrow,
            key_id: Some(key_id),
            x: center_mm.0,
            y: center_mm.1,
            width_u: 1.0,
            layer: 0,
        };

        geom.key_placements
            .insert(format!("{:?}", key_id), placement);
    }
}

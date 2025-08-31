use crate::{
    constants::{U2CELL, U2MM},
    error::Result,
    geometry::Geometry,
    geometry::types::BlockId,
    keys::{KeyId, allowed_widths},
    optimize::fitts::{FingerwiseFittsCoefficients, compute_fitts_time},
};
use std::collections::HashMap;

/// docs/v1.md Section 2.3に対応: 事前計算されたFitts時間
/// T(r, i, s)は特定のキーに依存せず、(r, i, s)の組み合わせのみで決まる
#[derive(Debug, Clone)]
pub struct PrecomputedFitts {
    /// 通常キー候補のFitts時間: (r, i, s) -> T(r, i, s) [ms]
    /// r: 行インデックス, i: 開始セル位置, s: 幅（セル単位）
    pub candidates: HashMap<(usize, usize, usize), f32>,

    /// 矢印・数字用1uブロックのFitts時間: block_id -> T_tap(u) [ms]
    pub blocks: HashMap<BlockId, f32>,
}

/// docs/v1.md Section 3.1に対応: 通常キー候補集合Cの生成と事前計算
/// C = {(r, i, s) | r ∈ R, s ∈ S, 0 ≤ i ≤ C - s, 空間制約満足}
pub fn precompute_fitts_times(
    geom: &Geometry,
    coeffs: &FingerwiseFittsCoefficients,
) -> Result<PrecomputedFitts> {
    let mut candidates = HashMap::new();
    let mut blocks = HashMap::new();

    // docs/v1.md Section 1.1: 可能な横幅の集合 S = {4, 5, 6, ..., 12}
    let width_range_cells = 4..=12; // 1u〜3u (セル単位)

    // 通常キー候補の事前計算
    // r ∈ R (行の集合)
    for r in 0..geom.cells.len() {
        // s ∈ S (可能な横幅の集合)
        for s in width_range_cells.clone() {
            // i の範囲: 0 ≤ i ≤ C - s
            let max_i = if geom.cells[r].len() >= s {
                geom.cells[r].len() - s
            } else {
                continue;
            };

            for i in 0..=max_i {
                // 空間制約チェック: O_{rj} = 0 ∀j ∈ [i, i + s - 1]
                let all_cells_free = (i..i + s).all(|j| !geom.cells[r][j].occupied);

                if all_cells_free {
                    // docs/v1.md Section 2.1: 中心座標計算
                    // x = (i + s/2) * Δ_mm, y = y_r
                    let center_mm = compute_candidate_center(r, i, s);

                    // docs/v1.md Section 2.3: 担当指の決定
                    // f = f(r, i + ⌊s/2⌋)
                    let center_col = i + s / 2;
                    let finger = geom.cells[r][center_col].finger;
                    let home_pos = geom.homes.get(&finger).copied().unwrap_or(center_mm);

                    // docs/v1.md Section 2.3: Fitts時間計算
                    // T(r, i, s) = a_f + b_f * log_2(D/W_eff + 1)
                    let width_u = s as f32 / U2CELL as f32;
                    let fitts_time_ms =
                        compute_fitts_time(finger, center_mm, home_pos, width_u, coeffs)?;

                    candidates.insert((r, i, s), fitts_time_ms);
                }
            }
        }
    }

    // 1uブロック（矢印キー用）の事前計算
    for row in 0..geom.cells.len() {
        let mut col = 0;
        while col + U2CELL <= geom.cells[row].len() {
            // 4セル全てが空きかチェック
            let all_cells_free = (col..col + U2CELL).all(|c| !geom.cells[row][c].occupied);

            if all_cells_free {
                let block_id = BlockId::new(row, col / U2CELL);
                let center_u = (col as f32 / U2CELL as f32 + 0.5, row as f32 + 0.5);
                let center_mm = (center_u.0 * U2MM, center_u.1 * U2MM);

                // ブロック中心の担当指
                let finger = geom.cells[row][col].finger;
                let home_pos = geom.homes.get(&finger).copied().unwrap_or(center_mm);

                let fitts_time_ms = compute_fitts_time(
                    finger, center_mm, home_pos, 1.0, // 1u固定
                    coeffs,
                )?;

                blocks.insert(block_id, fitts_time_ms);
            }
            col += U2CELL;
        }
    }

    log::info!(
        "事前計算完了: 通常キー候補 {}, 1uブロック {}",
        candidates.len(),
        blocks.len()
    );

    Ok(PrecomputedFitts { candidates, blocks })
}

/// docs/v1.md Section 2.1に対応: キー中心座標の計算
/// x = (i + s/2) * Δ_mm, y = y_r
fn compute_candidate_center(r: usize, i: usize, s: usize) -> (f32, f32) {
    // docs/v1.md Section 1.1: セルからmmへの変換係数 Δ_mm = d_mm/4
    let delta_mm = U2MM / U2CELL as f32;
    let x = (i as f32 + s as f32 / 2.0) * delta_mm;
    let y = (r as f32 + 0.5) * U2MM; // 行の中心
    (x, y)
}

/// docs/v1.md Section 3.1対応: 実際に配置可能な(r,i,s)候補を取得
/// 特定のキーに対する幅制約も考慮
pub fn get_valid_candidates_for_key(
    key: &KeyId,
    precomputed: &PrecomputedFitts,
) -> Vec<(usize, usize, usize, f32)> {
    let allowed_width_u = allowed_widths(key);
    let mut valid_candidates = Vec::new();

    for (&(r, i, s), &fitts_time) in precomputed.candidates.iter() {
        let width_u = s as f32 / U2CELL as f32;

        // キー固有の幅制約チェック
        if allowed_width_u.contains(&width_u) {
            valid_candidates.push((r, i, s, fitts_time));
        }
    }

    valid_candidates
}

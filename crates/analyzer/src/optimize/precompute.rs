use crate::{
    config::Config,
    constants::{DEFAULT_FKEYS_MAX, MAX_WIDTH_CELLS, MIN_WIDTH_CELLS, U2CELL, U2MM},
    error::{KbOptError, Result},
    geometry::{Geometry, types::finger_to_string},
    keys::{KeyId, SymbolKey},
    optimize::fitts::{FingerwiseFittsCoefficients, compute_fitts_time},
};
use std::collections::HashMap;

/// T(r, i, s)は特定のキーに依存せず、(r, i, s)の組み合わせのみで決まる
#[derive(Debug, Clone)]
pub struct PrecomputedFitts {
    /// 通常キー候補のFitts時間: (r, i, s) -> T(r, i, s) [ms]
    /// r: 行インデックス, i: 開始セル位置, s: 幅（セル単位）
    pub candidates: HashMap<(usize, usize, usize), f64>,
}

/// 通常キー候補集合Cの生成と事前計算
/// C = {(r, i, s) | r ∈ R, s ∈ S, 0 ≤ i ≤ C - s, 空間制約満足}
pub fn precompute_fitts_times(
    geom: &Geometry,
    coeffs: &FingerwiseFittsCoefficients,
) -> Result<PrecomputedFitts> {
    let mut candidates = HashMap::new();

    // 可能な横幅の集合 S = {4, 5, 6, ..., 12} (セル単位)
    let width_range_cells = MIN_WIDTH_CELLS..=MAX_WIDTH_CELLS;

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
                    // 中心座標計算
                    // x = (i + s/2) * Δ_mm, y = y_r
                    let center_mm = compute_candidate_center(r, i, s);

                    // 担当指の決定
                    // f = f(r, i + ⌊s/2⌋)
                    let center_col = i + s / 2;
                    let finger = geom.cells[r][center_col].finger;
                    let home_pos =
                        geom.homes
                            .get(&finger)
                            .copied()
                            .ok_or(KbOptError::Solver(format!(
                                "failed to get home position, finger: {}",
                                finger_to_string(&finger)
                            )))?;

                    // Fitts時間計算
                    // T(r, i, s) = a_f + b_f * log_2(D/W_eff + 1)
                    let fitts_time_ms = compute_fitts_time(finger, center_mm, home_pos, s, coeffs)?;

                    candidates.insert((r, i, s), fitts_time_ms);
                }
            }
        }
    }

    log::info!(
        "complete T(r, j, s) caliculation: candidate {}",
        candidates.len()
    );

    Ok(PrecomputedFitts { candidates })
}

/// キー中心座標の計算
/// x = (i + s/2) * Δ_mm, y = y_r
fn compute_candidate_center(r: usize, i: usize, s: usize) -> (f64, f64) {
    // セルからmmへの変換係数 Δ_mm = d_mm/4
    let delta_mm = U2MM / U2CELL as f64;
    let x = (i as f64 + s as f64 / 2.0) * delta_mm;
    let y = (r as f64 + 0.5) * U2MM; // 行の中心
    (x, y)
}

/// 最適化対象となるキーの一覧を取得
/// 矢印キーは別処理をするのでここでは含めない
pub fn all_movable_keys(config: &Config) -> Vec<KeyId> {
    use KeyId::*;
    use SymbolKey::*;

    let mut v = Vec::new();

    // include_alphabetがtrueの場合、アルファベットも最適化候補に入れる
    if config.solver.include_alphabet {
        use crate::keys::LetterKey::*;
        for letter in [
            A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
        ] {
            v.push(Letter(letter));
        }
    }

    // include_digitがtrueの場合、最適化候補に入れる
    if config.solver.include_digits {
        for d in 0..=9 {
            v.push(Digit(d));
        }
    }

    for s in [
        Backtick, Minus, Equal, LBracket, RBracket, Backslash, Semicolon, Quote, Comma, Period,
        Slash,
    ] {
        v.push(Symbol(s));
    }
    v.extend([
        Tab, Escape, CapsLock, Delete, Backspace, Space, Enter, ShiftL, ShiftR, CtrlL, CtrlR, AltL,
        AltR, MetaL, MetaR,
    ]);

    if config.solver.include_fkeys {
        for n in 1..=DEFAULT_FKEYS_MAX {
            v.push(Function(n));
        }
    }

    //if opt.include_modifiers {
    //    v.extend([
    //        Modifier(ModifierKey::Layer1),
    //        Modifier(ModifierKey::Layer2),
    //        Modifier(ModifierKey::Layer3),
    //    ]);
    //}

    //if opt.include_navigation {
    //    v.extend([Home, End, PageUp, PageDown, Insert]);
    //}
    //if opt.include_numpad {
    //    for n in 0..=9 {
    //        v.push(NumpadDigit(n));
    //    }
    //    v.extend([
    //        NumpadAdd,
    //        NumpadSubtract,
    //        NumpadMultiply,
    //        NumpadDivide,
    //        NumpadEnter,
    //        NumpadEquals,
    //        NumpadDecimal,
    //    ]);
    //}
    v
}

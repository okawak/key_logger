// v1: シンプルな単一係数Fitts法則実装

use crate::constants::{U2MM, euclid_distance};

/// v1: シンプルな単一係数Fitts法則
///
/// パラメータ:
/// - distance_mm: ホームポジションからの距離（mm）
/// - width_mm: ターゲット幅（mm）
/// - a_ms: Fitts係数a（ms）
/// - b_ms: Fitts係数b（ms）
pub fn compute_fitts_time(distance_mm: f64, width_mm: f64, a_ms: f64, b_ms: f64) -> f64 {
    a_ms + b_ms * ((distance_mm / width_mm + 1.0).log2())
}

/// v1: キー配置のFitts時間を計算
///
/// パラメータ:
/// - key_center: キー中心座標（mm単位）
/// - home_position: ホームポジション（mm単位）
/// - key_width_u: キー幅（u単位）
/// - a_ms: Fitts係数a（ms）
/// - b_ms: Fitts係数b（ms）
pub fn compute_key_fitts_time(
    key_center: (f32, f32),
    home_position: (f32, f32),
    key_width_u: f32,
    a_ms: f64,
    b_ms: f64,
) -> f64 {
    // 両方の座標がmm単位なので直接距離計算
    let distance_mm = euclid_distance(key_center, home_position) as f64;
    let width_mm = key_width_u as f64 * U2MM;

    compute_fitts_time(distance_mm, width_mm, a_ms, b_ms)
}

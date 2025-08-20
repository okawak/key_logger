/// # v1 Fitts Law Implementation (Enhanced)
///
/// CLAUDE.md v1仕様に基づくFitts時間計算:
/// - 基本: 単一係数Fitts法則（既存実装）
/// - 拡張: 指別係数対応
/// - 拡張: 方向依存の有効幅（楕円近似）
/// - 拡張: 端狙い補正（オプション）
use crate::constants::{U2MM, euclid_distance};
use crate::geometry::types::Finger;
use std::collections::HashMap;

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

// === v1 Enhanced Functions (CLAUDE.md v1 spec) ===

/// v1 Enhanced: 指別Fitts係数設定
#[derive(Debug, Clone)]
pub struct FingerwiseFittsCoefficients {
    /// 指別係数マップ: 指 → (a_ms, b_ms)
    pub coefficients: HashMap<Finger, (f64, f64)>,
    /// 方向依存幅の有効化
    pub enable_directional_width: bool,
}

impl FingerwiseFittsCoefficients {
    /// 設定から指別Fitts係数を作成
    pub fn from_config(config: &crate::optimize::config::FittsCoefficientsConfig) -> Self {
        let mut coefficients = HashMap::new();

        if let Some(ref values) = config.values {
            for (finger_str, coeff_array) in values {
                if let Some(finger) = crate::optimize::config::finger_from_string(finger_str) {
                    coefficients.insert(finger, (coeff_array[0], coeff_array[1]));
                }
            }
        }

        // 設定にない指はデフォルト値を使用
        let default_coeffs = Self::default();
        for (&finger, &default_coeff) in &default_coeffs.coefficients {
            coefficients.entry(finger).or_insert(default_coeff);
        }

        Self {
            coefficients,
            enable_directional_width: config.enable, // 設定の有効化フラグを使用
        }
    }
}

impl Default for FingerwiseFittsCoefficients {
    /// CLAUDE.mdの初期値に基づく指別係数
    fn default() -> Self {
        let mut coefficients = HashMap::new();

        // 人差し指: 最速
        coefficients.insert(Finger::LIndex, (40.0, 120.0));
        coefficients.insert(Finger::RIndex, (40.0, 120.0));

        // 中指
        coefficients.insert(Finger::LMiddle, (45.0, 130.0));
        coefficients.insert(Finger::RMiddle, (45.0, 130.0));

        // 薬指
        coefficients.insert(Finger::LRing, (55.0, 145.0));
        coefficients.insert(Finger::RRing, (55.0, 145.0));

        // 小指: 最遅
        coefficients.insert(Finger::LPinky, (65.0, 160.0));
        coefficients.insert(Finger::RPinky, (65.0, 160.0));

        // 親指
        coefficients.insert(Finger::LThumb, (50.0, 140.0));
        coefficients.insert(Finger::RThumb, (50.0, 140.0));

        Self {
            coefficients,
            enable_directional_width: true,
        }
    }
}

impl FingerwiseFittsCoefficients {
    /// 指の係数を取得
    pub fn get_coeffs(&self, finger: Finger) -> (f64, f64) {
        self.coefficients
            .get(&finger)
            .copied()
            .unwrap_or((50.0, 140.0)) // デフォルト値
    }
}

/// 共通化されたFitts時間計算（指別係数対応）
///
/// CLAUDE.md v1仕様:
/// ```
/// D = s_u2mm * ||x - h_f||_2
/// W = s_u2mm * W_eff(w, 1, φ)
/// T_tap = a_f + b_f * log2(D/W + 1)
/// ```
pub fn compute_fingerwise_fitts_time(
    finger: Finger,
    key_center_mm: (f32, f32),
    home_position_mm: (f32, f32),
    key_width_u: f32,
    coeffs: &FingerwiseFittsCoefficients,
) -> f64 {
    // 1. 距離計算
    let distance_mm = euclid_distance(key_center_mm, home_position_mm) as f64;

    // 2. 有効幅計算
    let effective_width_u = if coeffs.enable_directional_width {
        // 方向角計算
        let dx = key_center_mm.0 - home_position_mm.0;
        let dy = key_center_mm.1 - home_position_mm.1;
        let direction_angle = dy.atan2(dx);

        compute_directional_effective_width(key_width_u, 1.0, direction_angle)
    } else {
        key_width_u // 方向依存なし
    };

    let effective_width_mm = effective_width_u as f64 * U2MM;

    // 3. 指別Fitts時間計算
    let (a_f, b_f) = coeffs.get_coeffs(finger);
    compute_fitts_time(distance_mm, effective_width_mm, a_f, b_f)
}

/// 標準的なFitts時間計算（単一係数版との統合）
///
/// 全バージョン共通の計算処理
#[allow(clippy::too_many_arguments)]
pub fn compute_unified_fitts_time(
    finger: Finger,
    key_center_mm: (f32, f32),
    home_position_mm: (f32, f32),
    key_width_u: f32,
    use_fingerwise: bool,
    fingerwise_coeffs: &FingerwiseFittsCoefficients,
    default_a_ms: f64,
    default_b_ms: f64,
) -> f64 {
    if use_fingerwise {
        compute_fingerwise_fitts_time(
            finger,
            key_center_mm,
            home_position_mm,
            key_width_u,
            fingerwise_coeffs,
        )
    } else {
        compute_key_fitts_time(
            key_center_mm,
            home_position_mm,
            key_width_u,
            default_a_ms,
            default_b_ms,
        )
    }
}

/// 方向依存の有効幅計算（楕円近似）
///
/// CLAUDE.md v1仕様:
/// ```
/// W_eff(w, h, φ) = 1 / sqrt((cos²φ/w²) + (sin²φ/h²))
/// ```
pub fn compute_directional_effective_width(
    width_u: f32,
    height_u: f32,
    direction_angle: f32,
) -> f32 {
    let cos_phi = direction_angle.cos();
    let sin_phi = direction_angle.sin();

    let cos2_over_w2 = (cos_phi * cos_phi) / (width_u * width_u);
    let sin2_over_h2 = (sin_phi * sin_phi) / (height_u * height_u);

    1.0 / (cos2_over_w2 + sin2_over_h2).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_fitts_unchanged() {
        // 既存の基本Fitts関数が変更されていないことを確認
        let time = compute_fitts_time(100.0, 20.0, 50.0, 150.0);
        assert!(time > 0.0);
    }

    #[test]
    fn test_directional_width_horizontal() {
        // 水平移動（φ=0）では幅そのものになる
        let width = compute_directional_effective_width(2.0, 1.0, 0.0);
        assert!((width - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_directional_width_vertical() {
        // 垂直移動（φ=π/2）では高さそのものになる
        let width = compute_directional_effective_width(2.0, 1.0, std::f32::consts::PI / 2.0);
        assert!((width - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_fingerwise_coefficients() {
        let coeffs = FingerwiseFittsCoefficients::default();

        // 人差し指は最速
        let (a_index, b_index) = coeffs.get_coeffs(Finger::LIndex);
        assert_eq!(a_index, 40.0);
        assert_eq!(b_index, 120.0);

        // 小指は最遅
        let (a_pinky, b_pinky) = coeffs.get_coeffs(Finger::LPinky);
        assert_eq!(a_pinky, 65.0);
        assert_eq!(b_pinky, 160.0);
    }

    #[test]
    fn test_fingerwise_fitts_calculation() {
        let coeffs = FingerwiseFittsCoefficients::default();

        let key_center = (100.0, 50.0);
        let home_pos = (0.0, 0.0);
        let width = 1.0;

        // 人差し指と小指で時間差があることを確認
        let index_time = compute_unified_fitts_time(
            Finger::LIndex,
            key_center,
            home_pos,
            width,
            true,
            &coeffs,
            50.0,
            150.0,
        );
        let pinky_time = compute_unified_fitts_time(
            Finger::LPinky,
            key_center,
            home_pos,
            width,
            true,
            &coeffs,
            50.0,
            150.0,
        );

        // 小指の方が遅い
        assert!(pinky_time > index_time);

        // 統一関数のフラグテスト
        let legacy_time = compute_unified_fitts_time(
            Finger::LIndex,
            key_center,
            home_pos,
            width,
            false,
            &coeffs,
            50.0,
            150.0,
        );

        // レガシーモードでは指の違いは無視される
        assert!(
            (legacy_time - compute_key_fitts_time(key_center, home_pos, width, 50.0, 150.0)).abs()
                < 0.001
        );
    }
}

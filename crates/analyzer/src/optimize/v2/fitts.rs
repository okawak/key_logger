// Phase 1: 指別Fitts係数の実装準備
// このファイルは Phase 1 実装時に詳細化される

use crate::geometry::types::Finger;
use crate::optimize::config::{FittsCoefficientsConfig, finger_from_string};
use std::collections::HashMap;

/// 指別Fitts係数の設定
#[derive(Debug, Clone)]
pub struct FittsCoefficients {
    pub coeffs_per_finger: HashMap<Finger, (f64, f64)>, // (a_f, b_f)
}

impl Default for FittsCoefficients {
    fn default() -> Self {
        let mut coeffs = HashMap::new();

        // 研究ベースの初期値
        // 参考: MacKenzie, I. S., Marteniuk, R. G., Dugas, C., Liske, D., & Eickmeier, B. (1987).
        // "Three-dimensional movement trajectories in Fitts' task: Implications for control."
        // Quarterly Journal of Experimental Psychology, 39(4), 629-647. DOI:10.1080/14640748708401806
        //
        // 上記論文等を参考に、指ごとのFitts係数(a_f, b_f)の初期値を設定
        // 値は実験データに基づく暫定値であり、実際の使用時には調整が必要
        coeffs.insert(Finger::LThumb, (50.0, 150.0));
        coeffs.insert(Finger::LIndex, (40.0, 120.0));
        coeffs.insert(Finger::LMiddle, (45.0, 130.0));
        coeffs.insert(Finger::LRing, (55.0, 145.0));
        coeffs.insert(Finger::LPinky, (65.0, 160.0));
        coeffs.insert(Finger::RThumb, (50.0, 150.0));
        coeffs.insert(Finger::RIndex, (40.0, 120.0));
        coeffs.insert(Finger::RMiddle, (45.0, 130.0));
        coeffs.insert(Finger::RRing, (55.0, 145.0));
        coeffs.insert(Finger::RPinky, (65.0, 160.0));

        Self {
            coeffs_per_finger: coeffs,
        }
    }
}

impl FittsCoefficients {
    /// 設定から指別係数を作成
    /// Create finger-specific coefficients from configuration
    pub fn from_config(config: &FittsCoefficientsConfig) -> Self {
        if let Some(ref values) = config.values {
            let mut coeffs = HashMap::new();

            for (finger_str, coeffs_array) in values {
                if let Some(finger) = finger_from_string(finger_str) {
                    coeffs.insert(finger, (coeffs_array[0], coeffs_array[1]));
                } else {
                    log::warn!("Unknown finger name in config: {}", finger_str);
                }
            }

            // 不足している指はデフォルト値で補完
            let default_coeffs = Self::default();
            for (finger, default_coeff) in default_coeffs.coeffs_per_finger {
                coeffs.entry(finger).or_insert(default_coeff);
            }

            Self {
                coeffs_per_finger: coeffs,
            }
        } else {
            // values が None の場合はデフォルトを使用
            Self::default()
        }
    }
}

/// Phase 1: 指別のFitts時間計算
/// Finger-specific Fitts time calculation
pub fn compute_fitts_time_per_finger(
    finger: Finger,
    distance_mm: f64,
    width_mm: f64,
    coeffs: &FittsCoefficients,
) -> f64 {
    // Division by zero protection
    if width_mm <= 0.0 {
        log::error!(
            "Invalid width_mm ({}) for finger {:?}, using minimum value",
            width_mm,
            finger
        );
        return f64::INFINITY; // Return infinity for invalid input
    }

    // 指別の係数を取得
    let (a_f, b_f) = coeffs
        .coeffs_per_finger
        .get(&finger)
        .copied()
        .unwrap_or_else(|| {
            // フォールバック: デフォルト値を使用
            log::warn!(
                "No coefficients found for finger {:?}, using default values",
                finger
            );
            (50.0, 150.0)
        });

    // Fitts' law: T = a + b * log2(D/W + 1)
    a_f + b_f * ((distance_mm / width_mm + 1.0).log2())
}

/// Phase 2: 方向依存の有効幅計算（楕円近似）
/// Directional effective width calculation using elliptical approximation
///
/// 式: W_eff(w_u, φ) = 1/√((cos²φ/w_u²) + (sin²φ/h_u²))
/// Reference: Accot, J., & Zhai, S. (2002). More than dotting the i's—foundations for crossing-based interfaces.
pub fn effective_width_elliptical(w_u: f32, h_u: f32, direction_phi: f32) -> f32 {
    // 引数の検証
    if w_u <= 0.0 || h_u <= 0.0 {
        log::error!("Invalid key dimensions: w_u={}, h_u={}", w_u, h_u);
        return w_u.max(h_u).max(1.0); // フォールバック: より大きい方の値を使用
    }

    let cos_phi = direction_phi.cos();
    let sin_phi = direction_phi.sin();

    // 楕円近似計算: W_eff = 1 / sqrt(cos²φ/w_u² + sin²φ/h_u²)
    let denominator = (cos_phi * cos_phi) / (w_u * w_u) + (sin_phi * sin_phi) / (h_u * h_u);

    if denominator <= 0.0 {
        log::error!(
            "Invalid elliptical calculation denominator: {}",
            denominator
        );
        return w_u.max(h_u); // フォールバック
    }

    1.0 / denominator.sqrt()
}

/// Phase 2: 方向角の計算
/// Direction angle calculation from home position to key center
///
/// 戻り値: ラジアン単位の角度（-π から π）
/// - 0: 右方向（+X軸）
/// - π/2: 上方向（+Y軸）  
/// - π: 左方向（-X軸）
/// - -π/2: 下方向（-Y軸）
pub fn compute_direction_angle(from: (f32, f32), to: (f32, f32)) -> f32 {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;

    // 距離が非常に小さい場合（ほぼ同じ位置）
    if dx.abs() < 1e-6 && dy.abs() < 1e-6 {
        log::debug!(
            "Very small distance in direction calculation: dx={}, dy={}",
            dx,
            dy
        );
        return 0.0; // デフォルトは右方向
    }

    // atan2を使用して方向角を計算（-π から π の範囲）
    dy.atan2(dx)
}

/// Phase 1+2: 方向依存の指別Fitts時間計算
/// Directional finger-specific Fitts time calculation
///
/// Phase 1の指別係数とPhase 2の方向依存有効幅を組み合わせた計算
pub fn compute_fitts_time_directional(
    finger: Finger,
    distance_mm: f64,
    width_u: f32,
    direction_phi: f32,
    coeffs: &FittsCoefficients,
) -> f64 {
    // キー幅の検証
    if width_u <= 0.0 {
        log::error!(
            "Invalid width_u ({}) for directional Fitts calculation, finger {:?}",
            width_u,
            finger
        );
        return f64::INFINITY;
    }

    // Phase 2: 方向依存の有効幅を計算（h_u = 1.0u固定）
    let effective_width_u = effective_width_elliptical(width_u, 1.0, direction_phi);
    let effective_width_mm = effective_width_u as f64 * crate::constants::U2MM;

    // Phase 1: 指別係数を取得
    let (a_f, b_f) = coeffs
        .coeffs_per_finger
        .get(&finger)
        .copied()
        .unwrap_or_else(|| {
            log::warn!(
                "No coefficients found for finger {:?}, using default values",
                finger
            );
            (50.0, 150.0)
        });

    // Division by zero protection（Phase 1からの継承）
    if effective_width_mm <= 0.0 {
        log::error!(
            "Invalid effective_width_mm ({}) for finger {:?}, direction_phi={}",
            effective_width_mm,
            finger,
            direction_phi
        );
        return f64::INFINITY;
    }

    // Fitts' law with directional effective width: T = a + b * log2(D/W_eff + 1)
    a_f + b_f * ((distance_mm / effective_width_mm + 1.0).log2())
}

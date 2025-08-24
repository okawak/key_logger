use crate::{
    config::Config,
    constants::{U2MM, euclid_distance},
    error::{KbOptError, Result},
    geometry::types::{Finger, finger_from_string, finger_to_string},
};
use std::collections::HashMap;

/// Fittsの法則
pub fn fitts_law(distance_mm: f32, width_mm: f32, a_ms: f32, b_ms: f32) -> f32 {
    a_ms + b_ms * ((distance_mm / width_mm + 1.0).log2())
}

#[derive(Debug, Clone)]
pub struct FingerwiseFittsCoefficients {
    /// 指別係数マップ: 指 → (a_ms, b_ms)
    pub coefficients: HashMap<Finger, (f32, f32)>,
}

impl FingerwiseFittsCoefficients {
    /// 設定から指別Fitts係数を作成
    pub fn from_config(config: &Config) -> Self {
        let mut coefficients = HashMap::new();

        if let Some(ref values) = config.fingerwise_coeffs {
            for (finger_str, coeff) in values {
                if let Some(finger) = finger_from_string(finger_str) {
                    coefficients.insert(finger, (coeff.a_ms, coeff.b_ms));
                }
            }
        }

        // 設定にない指はデフォルト値を使用
        let default_coeffs = Self::default();
        for (&finger, &default_coeff) in &default_coeffs.coefficients {
            coefficients.entry(finger).or_insert(default_coeff);
        }

        Self { coefficients }
    }
}

impl Default for FingerwiseFittsCoefficients {
    fn default() -> Self {
        let mut coefficients = HashMap::new();

        // 人差し指
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

        Self { coefficients }
    }
}

impl FingerwiseFittsCoefficients {
    /// 指の係数を取得
    pub fn get_coeffs(&self, finger: Finger) -> Option<(f32, f32)> {
        self.coefficients.get(&finger).copied() // デフォルト値
    }
}

/// Fitts時間計算
pub fn compute_fitts_time(
    finger: Finger,
    key_center_mm: (f32, f32),
    home_position_mm: (f32, f32),
    key_width_u: f32,
    coeffs: &FingerwiseFittsCoefficients,
) -> Result<f32> {
    // 1. 距離計算
    let distance_mm = euclid_distance(key_center_mm, home_position_mm);

    // 2. 有効幅計算
    let effective_width_u = {
        // 方向角計算
        let dx = key_center_mm.0 - home_position_mm.0;
        let dy = key_center_mm.1 - home_position_mm.1;
        let direction_angle = dy.atan2(dx);

        compute_directional_effective_width(key_width_u, 1.0, direction_angle)
    };

    let effective_width_mm = effective_width_u * U2MM;

    // 3. 指別Fitts時間計算
    let (a_f, b_f) = coeffs.get_coeffs(finger).ok_or(KbOptError::Config(format!(
        "finger coefficient is not definded: {}",
        finger_to_string(finger)
    )))?;

    Ok(fitts_law(distance_mm, effective_width_mm, a_f, b_f))
}

/// 方向依存の有効幅計算（楕円近似）
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

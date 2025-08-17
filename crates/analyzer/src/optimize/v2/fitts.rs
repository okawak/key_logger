// Phase 1: 指別Fitts係数の実装準備
// このファイルは Phase 1 実装時に詳細化される

use crate::geometry::types::Finger;
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

/// Phase 1で実装予定: 指別のFitts時間計算
pub fn compute_fitts_time_per_finger(
    _finger: Finger,
    _distance_mm: f64,
    _width_mm: f64,
    _coeffs: &FittsCoefficients,
) -> f64 {
    // Phase 1で実装
    todo!("Phase 1: finger-specific Fitts time calculation not yet implemented")
}

/// Phase 2で実装予定: 方向依存の有効幅計算
pub fn effective_width_elliptical(_w_u: f32, _h_u: f32, _direction_phi: f32) -> f32 {
    // Phase 2で実装
    todo!("Phase 2: directional effective width not yet implemented")
}

/// Phase 2で実装予定: 方向角の計算
pub fn compute_direction_angle(_from: (f32, f32), _to: (f32, f32)) -> f32 {
    // Phase 2で実装
    todo!("Phase 2: direction angle calculation not yet implemented")
}

/// Phase 1+2で実装予定: 方向依存の指別Fitts時間計算
pub fn compute_fitts_time_directional(
    _finger: Finger,
    _distance_mm: f64,
    _width_u: f32,
    _direction_phi: f32,
    _coeffs: &FittsCoefficients,
) -> f64 {
    // Phase 1+2で実装
    todo!("Phase 1+2: directional finger-specific Fitts time not yet implemented")
}

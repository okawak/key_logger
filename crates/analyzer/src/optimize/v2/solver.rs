// v2ソルバー実装: 指別係数Fitts法則を使用した高度な最適化
// Phase 0では v1と同じ実装を使用（将来の段階的実装に向けた準備）

use super::SolveOptionsV2;
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;
use crate::geometry::Geometry;
use crate::optimize::{SolutionLayout, v1};

/// v2ソルバーのメインエントリポイント
///
/// Phase 0: v1互換実装
/// Phase 1+: 指別Fitts係数、方向依存幅などの拡張機能を段階的に追加
pub fn solve_layout_v2(
    geom: &mut Geometry,
    freqs: &KeyFreq,
    opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    // Phase 0: 基本チェックして、実装されていない機能に対してエラーを返す

    // Phase 1チェック
    if let Some(ref fitts_config) = opts.fitts_coeffs
        && fitts_config.enable
    {
        return Err(KbOptError::ConfigError(
            "Phase 1 (finger-specific Fitts coefficients) not yet implemented".to_string(),
        ));
    }

    // Phase 2チェック
    if let Some(ref directional_config) = opts.directional_width
        && directional_config.enable
    {
        return Err(KbOptError::ConfigError(
            "Phase 2 (directional effective width) not yet implemented".to_string(),
        ));
    }

    // Phase 3チェック
    if let Some(ref layers_config) = opts.layers
        && layers_config.enable
    {
        return Err(KbOptError::ConfigError(
            "Phase 3 (layer system) not yet implemented".to_string(),
        ));
    }

    // Phase 4チェック
    if let Some(ref digits_config) = opts.digits
        && digits_config.enable
    {
        return Err(KbOptError::ConfigError(
            "Phase 4 (digit cluster) not yet implemented".to_string(),
        ));
    }

    // Phase 5チェック
    if let Some(ref bigrams_config) = opts.bigrams
        && bigrams_config.enable
    {
        return Err(KbOptError::ConfigError(
            "Phase 5 (bigram approximation) not yet implemented".to_string(),
        ));
    }

    // Phase 0: すべての拡張機能が無効の場合、v1と同じ実装を使用
    println!("v2 solver: using v1 implementation (Phase 0)");
    v1::solve_layout_v1(geom, freqs, &opts.base)
}

/// Phase 1実装予定: 指別Fitts係数を使用したソルバー
pub fn solve_layout_with_finger_fitts(
    _geom: &mut Geometry,
    _freqs: &KeyFreq,
    _opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    todo!("Phase 1: finger-specific Fitts solver not yet implemented")
}

/// Phase 2実装予定: 方向依存幅を考慮したソルバー
pub fn solve_layout_with_directional_width(
    _geom: &mut Geometry,
    _freqs: &KeyFreq,
    _opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    todo!("Phase 2: directional width solver not yet implemented")
}

/// Phase 3実装予定: レイヤシステムを使用したソルバー
pub fn solve_layout_with_layers(
    _geom: &mut Geometry,
    _freqs: &KeyFreq,
    _opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    todo!("Phase 3: layer system solver not yet implemented")
}

/// Phase 4実装予定: 数値クラスターを考慮したソルバー
pub fn solve_layout_with_digit_cluster(
    _geom: &mut Geometry,
    _freqs: &KeyFreq,
    _opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    todo!("Phase 4: digit cluster solver not yet implemented")
}

/// Phase 5実装予定: ビグラム近似を使用したソルバー
pub fn solve_layout_with_bigrams(
    _geom: &mut Geometry,
    _freqs: &KeyFreq,
    _opts: &SolveOptionsV2,
) -> Result<SolutionLayout, KbOptError> {
    todo!("Phase 5: bigram approximation solver not yet implemented")
}

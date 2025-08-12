use super::types::*;
use std::collections::HashMap;

/// 小指端ルール
#[derive(Debug, Clone, Copy)]
pub enum PinkyEdgeRule {
    Off,
    /// すべての行で「小指ホーム x より外側は小指」
    AllRows,
    /// 指定行 r_idx 以上で適用（例：Bottom 段から）
    BelowRow {
        row_idx: usize,
    },
}

/// 境界の決め方
#[derive(Debug, Clone)]
pub enum BoundaryMode {
    /// ホーム段（ASDF 行）のキー中心から自動導出（標準的な配分）
    DerivedFromHomeRow,
    /// 手動（左→右の 8 境界）
    Manual([f32; 8]),
}

/// 指ゾーンの仕様
#[derive(Debug, Clone)]
pub struct ZonePolicy {
    pub mode: BoundaryMode,
    pub pinky_edge_rule: PinkyEdgeRule,
}

impl Default for ZonePolicy {
    fn default() -> Self {
        Self {
            mode: BoundaryMode::DerivedFromHomeRow,
            // ご要望に合わせて既定は「全行で端は小指」
            pinky_edge_rule: PinkyEdgeRule::AllRows,
        }
    }
}

/// ポリシーに基づき、セルごとの担当指を再割当
pub fn apply_zone_policy(geom: &mut Geometry, zp: &ZonePolicy) {
    // 1) 境界を決定
    let boundaries_8: [f32; 8] = match zp.mode {
        BoundaryMode::Manual(b) => b,
        BoundaryMode::DerivedFromHomeRow => derive_boundaries_from_homerow(geom),
    };

    // GeometryConfig へ [f32;9] に拡張して保存（最後は 15.0）
    let mut bounds9 = [0.0f32; 9];
    bounds9[0] = boundaries_8[0];
    bounds9[1] = boundaries_8[1];
    bounds9[2] = boundaries_8[2];
    bounds9[3] = boundaries_8[3];
    bounds9[4] = boundaries_8[4];
    bounds9[5] = boundaries_8[5];
    bounds9[6] = boundaries_8[6];
    bounds9[7] = boundaries_8[7];
    bounds9[8] = 15.0; // 右端
    geom.cfg.finger_x_boundaries = bounds9;

    // 2) ホームの座標（小指／親指で使用）
    let homes: &HashMap<Finger, (f32, f32)> = &geom.homes;
    let (lpx, _) = homes.get(&Finger::LPinky).cloned().unwrap_or((2.0, 0.0));
    let (rpx, _) = homes.get(&Finger::RPinky).cloned().unwrap_or((13.0, 0.0));
    let (ltx, _) = homes.get(&Finger::LThumb).cloned().unwrap_or((5.5, 0.0));
    let (rtx, _) = homes.get(&Finger::RThumb).cloned().unwrap_or((9.5, 0.0));
    let thumb_mid = (ltx + rtx) * 0.5;

    // 3) すべてのセルを再割当
    for r in 0..geom.cfg.rows.len() {
        let row_spec = &geom.cfg.rows[r];
        for c in 0..geom.cells_per_row {
            let cell = &mut geom.cells[r][c];
            let cx = row_spec.offset_u + (c as f32 + 0.5) * CELL_U;

            // まず境界ベースで割当
            let b = &geom.cfg.finger_x_boundaries;
            let mut fg = if cx < b[0] {
                Finger::LPinky
            } else if cx < b[1] {
                Finger::LRing
            } else if cx < b[2] {
                Finger::LMiddle
            } else if cx < b[3] {
                Finger::LIndex
            } else if cx < b[4] {
                Finger::RIndex
            } else if cx < b[5] {
                Finger::RMiddle
            } else if cx < b[6] {
                Finger::RRing
            } else {
                Finger::RPinky
            }; // 右端（b[6]..15.0）

            // 親指行は中央のみ親指、端は小指に上書き
            if r == geom.cfg.thumb_row {
                // 端の小指優先
                let apply_pinky = match zp.pinky_edge_rule {
                    PinkyEdgeRule::Off => false,
                    PinkyEdgeRule::AllRows => true,
                    PinkyEdgeRule::BelowRow { row_idx } => r >= row_idx,
                };
                if apply_pinky {
                    if cx <= lpx {
                        fg = Finger::LPinky;
                    }
                    if cx >= rpx {
                        fg = Finger::RPinky;
                    }
                }
                // 端ではない場合のみ親指へ
                if fg != Finger::LPinky && fg != Finger::RPinky {
                    fg = if cx < thumb_mid {
                        Finger::LThumb
                    } else {
                        Finger::RThumb
                    };
                }
            } else {
                // 親指行以外でも端は小指（要望に合わせ AllRows 既定）
                let apply_pinky = match zp.pinky_edge_rule {
                    PinkyEdgeRule::Off => false,
                    PinkyEdgeRule::AllRows => true,
                    PinkyEdgeRule::BelowRow { row_idx } => r >= row_idx,
                };
                if apply_pinky {
                    if cx <= lpx {
                        fg = Finger::LPinky;
                    }
                    if cx >= rpx {
                        fg = Finger::RPinky;
                    }
                }
            }

            cell.finger = fg;
        }
    }
}

/// ホーム段(ASDF)のキー中心から境界を導出
///
///  b0=mid(A,S), b1=mid(S,D), b2=mid(D,F), b3=mid(F,G),
///  b4=mid(H,J), b5=mid(J,K), b6=mid(K,L), b7=mid(L,;)
fn derive_boundaries_from_homerow(geom: &Geometry) -> [f32; 8] {
    // ASDF 行の列中心を計算（RowStagger では offset=1.75u）
    let row = 2usize; // Middle row
    let r = &geom.cfg.rows[row];

    // A キー左端(=スタート)は 1.75u として予約しているため、それに合わせる
    // （build::reserve_letter_blocks と整合）
    let start_u = 1.75_f32;

    let center = |n: i32| -> f32 {
        // n=0..9（A..;）を想定、ただし行オフセットは r.offset_u を尊重
        r.offset_u + (start_u - r.offset_u) + (n as f32 + 0.5) * ONE_U
    };
    let a = center(0);
    let s = center(1);
    let d = center(2);
    let f = center(3);
    let g = center(4);
    let h = center(5);
    let j = center(6);
    let k = center(7);
    let l = center(8);
    let semi = center(9);

    let mid = |x: f32, y: f32| -> f32 { 0.5 * (x + y) };

    [
        mid(a, s),    // b0  LPinky/LRing
        mid(s, d),    // b1  LRing/LMiddle
        mid(d, f),    // b2  LMiddle/LIndex
        mid(f, g),    // b3  LIndex/中心寄り（G 側）
        mid(h, j),    // b4  中心寄り（H 側）/RIndex
        mid(j, k),    // b5  RIndex/RMiddle
        mid(k, l),    // b6  RMiddle/RRing
        mid(l, semi), // b7  RRing/RPinky
    ]
}

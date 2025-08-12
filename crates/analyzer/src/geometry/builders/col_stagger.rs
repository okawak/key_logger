use super::super::types::*;
use super::GeometryBuilder;
use std::collections::HashMap;

pub struct ColStaggerBuilder;

impl GeometryBuilder for ColStaggerBuilder {
    fn build_rows(cells_per_row: usize) -> Vec<RowSpec> {
        vec![
            RowSpec {
                offset_u: 0.00,
                base_y_u: 0.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Number row
            RowSpec {
                offset_u: 0.00,
                base_y_u: 1.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Top row QWERTY
            RowSpec {
                offset_u: 0.00,
                base_y_u: 2.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Middle row ASDF
            RowSpec {
                offset_u: 0.00,
                base_y_u: 3.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Bottom row ZXCV
            RowSpec {
                offset_u: 0.00,
                base_y_u: 4.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Space row
        ]
    }

    fn build_col_stagger_y(cells_per_row: usize) -> Vec<f32> {
        let mut col_stagger_y = vec![0.0f32; cells_per_row];

        // ColStagger: 中指が最も高く、人差し指・薬指が0.25u下、小指がさらに0.25u下
        for (c, offset) in col_stagger_y.iter_mut().enumerate().take(cells_per_row) {
            let x_pos = c as f32 * CELL_U;
            // 指の境界を基に計算 (finger_x_boundaries参照)
            if x_pos < 5.5 {
                // Left pinky: -0.25u (最も下)
                *offset = -0.25;
            } else if x_pos < 7.0 {
                // Left ring: -0.25u
                *offset = -0.25;
            } else if x_pos < 8.75 {
                // Left middle: 0.0u (最も高い)
                *offset = 0.0;
            } else if x_pos < 10.5 {
                // Left index: -0.25u
                *offset = -0.25;
            } else if x_pos < 12.0 {
                // Right index: -0.25u
                *offset = -0.25;
            } else if x_pos < 13.5 {
                // Right middle: 0.0u (最も高い)
                *offset = 0.0;
            } else if x_pos < 15.0 {
                // Right ring: -0.25u
                *offset = -0.25;
            } else {
                // Right pinky: -0.5u (最も下)
                *offset = -0.5;
            }
        }

        col_stagger_y
    }

    fn get_letter_block_positions() -> Vec<(usize, f32, usize)> {
        vec![
            (1, 1.50, 10), // Top row QWERTY: 10 keys, start=1.50u (relative to no offset)
            (2, 1.75, 9),  // Middle row ASDF: 9 keys, start=1.75u (relative to no offset)
            (3, 2.25, 7),  // Bottom row ZXCV: 7 keys, start=2.25u (relative to no offset)
        ]
    }

    fn calculate_home_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        char_idx: usize,
    ) -> (f32, f32) {
        let r = &geometry_cfg.rows[row_idx];
        let a_start_col = cells_from_u((1.75 - r.offset_u).max(0.0));
        let start = a_start_col + char_idx * cells_from_u(ONE_U);
        let center_col = start + cells_from_u(ONE_U) / 2;
        let x = r.offset_u + (center_col as f32) * CELL_U;
        let mut y = geometry_cfg.rows[row_idx].base_y_u;

        // ColStaggerの場合は列オフセットを追加
        if center_col < geometry_cfg.col_stagger_y.len() {
            y += geometry_cfg.col_stagger_y[center_col];
        }

        (x, y)
    }

    fn get_fixed_key_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        col_idx: usize,
    ) -> (f32, f32) {
        let r = &geometry_cfg.rows[row_idx];
        let x0 = r.offset_u + col_idx as f32 * CELL_U;
        let mut y0 = r.base_y_u - 0.5;

        // ColStaggerの場合は列オフセットを追加
        let effective_col_idx = ((x0 - r.offset_u) / CELL_U) as usize;
        if effective_col_idx < geometry_cfg.col_stagger_y.len() {
            y0 += geometry_cfg.col_stagger_y[effective_col_idx];
        }

        (x0, y0)
    }

    fn get_qwerty_label_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        char_idx: usize,
    ) -> (f32, f32) {
        let r = &geometry_cfg.rows[row_idx];
        let x = r.offset_u + (char_idx as f32 + 0.5) * ONE_U;
        let mut y = r.base_y_u;

        // ColStaggerの場合は列オフセットを追加
        let col_idx = ((x - r.offset_u) / CELL_U) as usize;
        if col_idx < geometry_cfg.col_stagger_y.len() {
            y += geometry_cfg.col_stagger_y[col_idx];
        }

        (x, y)
    }

    fn build_home_positions(geometry_cfg: &GeometryConfig) -> HashMap<Finger, (f32, f32)> {
        let mut homes = HashMap::new();

        // ColStaggerの場合は列オフセットを考慮したホームポジション
        // Middle row=2での各指の列位置
        let row = 2usize;
        let base_y = geometry_cfg.rows[row].base_y_u;

        // 各指の列位置を定義（0.25u単位）
        let finger_positions = [
            (Finger::LPinky, 1.5 * ONE_U),  // 左小指
            (Finger::LRing, 2.5 * ONE_U),   // 左薬指
            (Finger::LMiddle, 3.5 * ONE_U), // 左中指（最も高い）
            (Finger::LIndex, 4.5 * ONE_U),  // 左人差し指
            (Finger::RIndex, 6.5 * ONE_U),  // 右人差し指
            (Finger::RMiddle, 7.5 * ONE_U), // 右中指（最も高い）
            (Finger::RRing, 8.5 * ONE_U),   // 右薬指
            (Finger::RPinky, 9.5 * ONE_U),  // 右小指
        ];

        for (finger, x) in finger_positions {
            // 列オフセットを適用
            let col_idx = (x / CELL_U) as usize;
            let y_offset = if col_idx < geometry_cfg.col_stagger_y.len() {
                geometry_cfg.col_stagger_y[col_idx]
            } else {
                0.0
            };
            let y = base_y + y_offset;
            homes.insert(finger, (x, y));
        }

        // 親指ポジション（列オフセット考慮）
        let thumb_y = geometry_cfg.rows[geometry_cfg.thumb_row].base_y_u;
        homes.insert(Finger::LThumb, (4.0 * ONE_U, thumb_y)); // 左親指
        homes.insert(Finger::RThumb, (7.0 * ONE_U, thumb_y)); // 右親指

        homes
    }
}

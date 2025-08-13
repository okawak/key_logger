use super::super::types::*;
use super::GeometryBuilder;
use std::collections::HashMap;

pub struct RowStaggerBuilder;

impl GeometryBuilder for RowStaggerBuilder {
    fn build_rows() -> Vec<RowSpec> {
        vec![
            RowSpec {
                offset_x: 0,
                offset_y: 0,
            }, // Space row (親指行) - 一番下
            RowSpec {
                offset_x: 0,
                offset_y: 4,
            }, // Bottom row ZXCV
            RowSpec {
                offset_x: 0,
                offset_y: 8,
            }, // Middle row ASDF
            RowSpec {
                offset_x: 0,
                offset_y: 12,
            }, // Top row QWERTY
            RowSpec {
                offset_x: 0,
                offset_y: 16,
            }, // Number row - 一番上
        ]
    }

    fn build_col_stagger_y(_cells_per_row: usize) -> Vec<f32> {
        // RowStaggerでは列オフセットは使用しない
        vec![]
    }

    fn get_letter_block_positions() -> Vec<(usize, f32, usize)> {
        vec![
            (3, 1.50, 10), // Top row QWERTY: 10 keys, start=1.50u (絶対位置)
            (2, 1.75, 9),  // Middle row ASDF: 9 keys, start=1.75u (絶対位置)
            (1, 2.25, 7),  // Bottom row ZXCV: 7 keys, start=2.25u (絶対位置)
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
        let y = geometry_cfg.rows[row_idx].base_y_u;
        (x, y)
    }

    fn get_fixed_key_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        col_idx: usize,
    ) -> (f32, f32) {
        // アルファベット文字の固定キー位置を計算（col_idxは文字インデックス0,1,2...）
        let alphabet_start_positions = [0.0, 2.25, 1.75, 1.50, 0.0]; // [row0(親指), row1(ZXCV), row2(ASDF), row3(QWERTY), row4(数字)]
        let alphabet_start_u = alphabet_start_positions[row_idx];

        // セル単位でのアルファベット開始位置計算
        let alphabet_start_cells = cells_from_u(alphabet_start_u);

        // col_idx番目のキーのセル開始位置（1u = 4セル）
        let key_start_cells = alphabet_start_cells + col_idx * cells_from_u(ONE_U);

        // 固定キー枠の位置（左端）を計算
        let x0 = key_start_cells as f32 * CELL_U;
        let y0 = geometry_cfg.rows[row_idx].base_y_u - 0.5;
        (x0, y0)
    }

    fn get_qwerty_label_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        char_idx: usize,
    ) -> (f32, f32) {
        // 固定キー位置と完全に一致させるため、同じ計算ロジックを使用
        let r = &geometry_cfg.rows[row_idx];

        // アルファベット開始位置（絶対座標u単位）
        let alphabet_start_positions = [0.0, 2.25, 1.75, 1.50, 0.0]; // [row0(親指), row1(ZXCV), row2(ASDF), row3(QWERTY), row4(数字)]
        let alphabet_start_u = alphabet_start_positions[row_idx];

        // セル単位でのアルファベット開始位置計算
        let alphabet_start_cells = cells_from_u(alphabet_start_u);

        // char_idx番目のキーのセル開始位置（1u = 4セル）
        let key_start_cells = alphabet_start_cells + char_idx * cells_from_u(ONE_U);

        // 固定キー枠の位置（左端）を計算 - get_fixed_key_positionと同じロジック
        let key_left_u = key_start_cells as f32 * CELL_U;

        // ラベル位置はキーの中心
        let key_center_u = key_left_u + ONE_U / 2.0;
        let y = r.base_y_u;

        (key_center_u, y)
    }

    fn build_home_positions(geometry_cfg: &GeometryConfig) -> HashMap<Finger, (f32, f32)> {
        let mut homes = HashMap::new();

        // RowStaggerの標準的なホームポジション（ASDF JKL;）
        // Middle row=2 starting from A position
        let row = 2usize;
        let r = &geometry_cfg.rows[row];
        let a_start_col = cells_from_u((1.75 - r.offset_u).max(0.0));

        let idx = |n: usize| -> (f32, f32) {
            let start = a_start_col + n * cells_from_u(ONE_U);
            let center_col = start + cells_from_u(ONE_U) / 2;
            let x = r.offset_u + (center_col as f32) * CELL_U;
            let y = geometry_cfg.rows[row].base_y_u;
            (x, y)
        };

        // 親指ポジション
        let thumb_y = geometry_cfg.rows[0].base_y_u;
        let lthumb = (5.5, thumb_y);
        let rthumb = (9.5, thumb_y);

        homes.insert(Finger::LPinky, idx(0)); // A
        homes.insert(Finger::LRing, idx(1)); // S
        homes.insert(Finger::LMiddle, idx(2)); // D
        homes.insert(Finger::LIndex, idx(3)); // F
        homes.insert(Finger::RIndex, idx(6)); // J
        homes.insert(Finger::RMiddle, idx(7)); // K
        homes.insert(Finger::RRing, idx(8)); // L
        let (lx, ly) = idx(8);
        homes.insert(Finger::RPinky, (lx + ONE_U, ly)); // ;
        homes.insert(Finger::LThumb, lthumb);
        homes.insert(Finger::RThumb, rthumb);

        homes
    }
}

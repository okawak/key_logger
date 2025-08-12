use super::super::types::*;
use super::GeometryBuilder;
use std::collections::HashMap;

pub struct RowStaggerBuilder;

impl GeometryBuilder for RowStaggerBuilder {
    fn build_rows(cells_per_row: usize) -> Vec<RowSpec> {
        vec![
            RowSpec {
                offset_u: 0.00,
                base_y_u: 0.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Number row
            RowSpec {
                offset_u: 1.50,
                base_y_u: 1.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Top row QWERTY
            RowSpec {
                offset_u: 1.75,
                base_y_u: 2.0,
                width_u: 15.0,
                cells: cells_per_row,
            }, // Middle row ASDF
            RowSpec {
                offset_u: 2.25,
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

    fn build_col_stagger_y(_cells_per_row: usize) -> Vec<f32> {
        // RowStaggerでは列オフセットは使用しない
        vec![]
    }

    fn get_letter_block_positions() -> Vec<(usize, f32, usize)> {
        vec![
            (1, 1.50, 10), // Top row QWERTY: 10 keys, start=1.50u
            (2, 1.75, 9),  // Middle row ASDF: 9 keys, start=1.75u
            (3, 2.25, 7),  // Bottom row ZXCV: 7 keys, start=2.25u
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
        let r = &geometry_cfg.rows[row_idx];
        let x0 = r.offset_u + col_idx as f32 * CELL_U;
        let y0 = r.base_y_u - 0.5;
        (x0, y0)
    }

    fn get_qwerty_label_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        char_idx: usize,
    ) -> (f32, f32) {
        let r = &geometry_cfg.rows[row_idx];
        let x = r.offset_u + (char_idx as f32 + 0.5) * ONE_U;
        let y = r.base_y_u;
        (x, y)
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
        let thumb_y = geometry_cfg.rows[geometry_cfg.thumb_row].base_y_u;
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
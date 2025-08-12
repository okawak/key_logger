use super::super::types::*;
use super::GeometryBuilder;
use std::collections::HashMap;

pub struct OrthoBuilder;

impl GeometryBuilder for OrthoBuilder {
    fn build_rows(cells_per_row: usize) -> Vec<RowSpec> {
        vec![
            RowSpec {
                offset_u: 0.00,
                base_y_u: 0.0,
                width_u: 15.0,
                cells: cells_per_row,
            },
            RowSpec {
                offset_u: 0.00,
                base_y_u: 1.0,
                width_u: 15.0,
                cells: cells_per_row,
            },
            RowSpec {
                offset_u: 0.00,
                base_y_u: 2.0,
                width_u: 15.0,
                cells: cells_per_row,
            },
            RowSpec {
                offset_u: 0.00,
                base_y_u: 3.0,
                width_u: 15.0,
                cells: cells_per_row,
            },
            RowSpec {
                offset_u: 0.00,
                base_y_u: 4.0,
                width_u: 15.0,
                cells: cells_per_row,
            },
        ]
    }

    fn build_col_stagger_y(_cells_per_row: usize) -> Vec<f32> {
        // Orthoでは列オフセットは使用しない
        vec![]
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
        let y = geometry_cfg.rows[row_idx].base_y_u;
        (x, y)
    }

    fn get_fixed_key_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        col_idx: usize,
    ) -> (f32, f32) {
        // Orthoの場合は行オフセットを無視
        let x0 = col_idx as f32 * CELL_U;
        let y0 = geometry_cfg.rows[row_idx].base_y_u - 0.5;
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
        
        // Orthoの場合は格子配列なので、より均等で効率的なホームポジション
        // Middle row=2での格子位置基準
        let row = 2usize;
        let base_y = geometry_cfg.rows[row].base_y_u;
        
        // 左手ホーム位置（格子上でより自然な位置）
        homes.insert(Finger::LPinky, (1.5 * ONE_U, base_y)); // 格子上の1.5u位置
        homes.insert(Finger::LRing, (2.5 * ONE_U, base_y));  // 格子上の2.5u位置
        homes.insert(Finger::LMiddle, (3.5 * ONE_U, base_y)); // 格子上の3.5u位置
        homes.insert(Finger::LIndex, (4.5 * ONE_U, base_y)); // 格子上の4.5u位置
        
        // 右手ホーム位置（格子上でより自然な位置）
        homes.insert(Finger::RIndex, (6.5 * ONE_U, base_y)); // 格子上の6.5u位置
        homes.insert(Finger::RMiddle, (7.5 * ONE_U, base_y)); // 格子上の7.5u位置
        homes.insert(Finger::RRing, (8.5 * ONE_U, base_y)); // 格子上の8.5u位置
        homes.insert(Finger::RPinky, (9.5 * ONE_U, base_y)); // 格子上の9.5u位置
        
        // 親指ポジション（格子上での対称的配置）
        let thumb_y = geometry_cfg.rows[geometry_cfg.thumb_row].base_y_u;
        homes.insert(Finger::LThumb, (4.0 * ONE_U, thumb_y)); // 左親指
        homes.insert(Finger::RThumb, (7.0 * ONE_U, thumb_y)); // 右親指
        
        homes
    }
}
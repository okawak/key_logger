use std::collections::HashMap;

use super::{
    builders::{
        GeometryBuilder, col_stagger::ColStaggerBuilder, ortho::OrthoBuilder,
        row_stagger::RowStaggerBuilder,
    },
    types::*,
    zoning::{ZonePolicy, apply_zone_policy},
};
use crate::error::Result;

/// Geometry construction: 0.25u grid, fixed letters reservation, homes
/// 指ゾーンの最終決定は zoning::apply_zone_policy に委譲する
impl Geometry {
    /// デフォルトのゾーンポリシーで構築
    pub fn build(name: GeometryName) -> Result<Self> {
        let zp = ZonePolicy::default();
        Self::build_with_zone(name, &zp)
    }

    /// 任意のゾーンポリシーで構築（可視化/実験用途）
    pub fn build_with_zone(name: GeometryName, zp: &ZonePolicy) -> Result<Self> {
        // 5 rows × 15u (= 60 cells/row)
        let cells_per_row = 60usize;

        // Row specifications and column offsets using builder pattern
        let (rows, col_stagger_y) = match name {
            GeometryName::RowStagger => (
                RowStaggerBuilder::build_rows(cells_per_row),
                RowStaggerBuilder::build_col_stagger_y(cells_per_row),
            ),
            GeometryName::Ortho => (
                OrthoBuilder::build_rows(cells_per_row),
                OrthoBuilder::build_col_stagger_y(cells_per_row),
            ),
            GeometryName::ColStagger => (
                ColStaggerBuilder::build_rows(cells_per_row),
                ColStaggerBuilder::build_col_stagger_y(cells_per_row),
            ),
        };

        // 初期の指境界（暫定）— 最終値は apply_zone_policy が上書きする
        // ※ vis.rs のバンド描画は build() 戻り値を使う時点では apply 後の値になる
        let finger_x_boundaries = [3.5, 5.5, 7.0, 8.75, 10.5, 12.0, 13.5, 15.0, 15.0];

        let cfg = GeometryConfig {
            cell_pitch_u: CELL_U,
            rows: rows.clone(),
            col_stagger_y,
            finger_x_boundaries,
            thumb_row: 4,
        };

        // Cell generation（finger は暫定。最後に apply_zone_policy が確定させる）
        let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(rows.len());
        for (r_idx, r) in rows.iter().enumerate() {
            let mut row_cells = Vec::with_capacity(r.cells);
            for c in 0..r.cells {
                let x = r.offset_u + (c as f32 + 0.5) * CELL_U;
                let mut y = r.base_y_u;
                if matches!(name, GeometryName::ColStagger) {
                    y += cfg.col_stagger_y[c];
                }
                // 暫定：親指行だけ左右で親指、それ以外は境界で推定
                let finger = if r_idx == cfg.thumb_row {
                    if x < 7.5 {
                        Finger::LThumb
                    } else {
                        Finger::RThumb
                    }
                } else {
                    finger_from_x(x, &cfg.finger_x_boundaries)
                };
                row_cells.push(Cell {
                    id: CellId::new(r_idx, c),
                    center_x_u: x,
                    center_y_u: y,
                    finger,
                    fixed_occupied: false,
                });
            }
            cells.push(row_cells);
        }

        let mut geom = Geometry {
            name,
            cfg,
            cells,
            homes: HashMap::new(),
            cells_per_row,
        };

        // 固定文字（A..Z）を確保
        geom.reserve_letter_blocks();
        // ホーム位置（ASDF / JKL;）
        geom.init_homes();

        // ★ 最後にゾーンポリシーを適用（最終境界と担当指をここで確定）
        apply_zone_policy(&mut geom, zp);

        Ok(geom)
    }

    /// Reserve letter blocks (using builder pattern)
    fn reserve_letter_blocks(&mut self) {
        let positions = match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::get_letter_block_positions(),
            GeometryName::Ortho => OrthoBuilder::get_letter_block_positions(),
            GeometryName::ColStagger => ColStaggerBuilder::get_letter_block_positions(),
        };

        for (row_idx, start_u, count_1u) in positions {
            self.reserve_run(row_idx, start_u, count_1u);
        }
    }
    fn reserve_run(&mut self, row: usize, start_u: f32, count_1u: usize) {
        // start_uは絶対座標なので、行のオフセットは引かない
        let start_col = cells_from_u(start_u.max(0.0));
        for k in 0..count_1u {
            let i = start_col + k * cells_from_u(ONE_U);
            for c in i..(i + cells_from_u(ONE_U)) {
                if c < self.cells_per_row {
                    self.cells[row][c].fixed_occupied = true;
                }
            }
        }
    }

    /// Home positions (geometry-specific)
    fn init_homes(&mut self) {
        self.homes = match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::build_home_positions(&self.cfg),
            GeometryName::Ortho => OrthoBuilder::build_home_positions(&self.cfg),
            GeometryName::ColStagger => ColStaggerBuilder::build_home_positions(&self.cfg),
        };
    }

    /// Calculate geometry-aware position for fixed key rectangles
    pub fn get_fixed_key_position(&self, row_idx: usize, col_idx: usize) -> (f32, f32) {
        match self.name {
            GeometryName::RowStagger => {
                RowStaggerBuilder::get_fixed_key_position(&self.cfg, row_idx, col_idx)
            }
            GeometryName::Ortho => {
                OrthoBuilder::get_fixed_key_position(&self.cfg, row_idx, col_idx)
            }
            GeometryName::ColStagger => {
                ColStaggerBuilder::get_fixed_key_position(&self.cfg, row_idx, col_idx)
            }
        }
    }

    /// Calculate geometry-aware position for QWERTY labels
    pub fn get_qwerty_label_position(&self, row_idx: usize, char_idx: usize) -> (f32, f32) {
        match self.name {
            GeometryName::RowStagger => {
                RowStaggerBuilder::get_qwerty_label_position(&self.cfg, row_idx, char_idx)
            }
            GeometryName::Ortho => {
                OrthoBuilder::get_qwerty_label_position(&self.cfg, row_idx, char_idx)
            }
            GeometryName::ColStagger => {
                ColStaggerBuilder::get_qwerty_label_position(&self.cfg, row_idx, char_idx)
            }
        }
    }
}

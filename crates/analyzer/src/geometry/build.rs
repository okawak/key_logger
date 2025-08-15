use std::collections::HashMap;

use super::{
    builders::{GeometryBuilder, ortho::OrthoBuilder, row_stagger::RowStaggerBuilder},
    types::*,
    zoning::finger_from_x,
};
use crate::constants::{MAX_COL_CELLS, MAX_ROW, U2CELL, cell_to_key_center};
use crate::error::Result;

/// Geometry construction: 0.25u grid, fixed letters reservation, homes
impl Geometry {
    pub fn build(name: GeometryName) -> Result<Self> {
        let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(MAX_ROW);
        for row in 0..MAX_ROW {
            let mut row_cells = Vec::with_capacity(MAX_COL_CELLS);
            for col in 0..MAX_COL_CELLS {
                let finger = if row == 0 {
                    // 一番下の行
                    // 小指のx領域は親指領域でも小指が担当
                    let finger_by_x = finger_from_x(col);
                    if matches!(finger_by_x, Finger::LPinky | Finger::RPinky) {
                        finger_by_x
                    } else {
                        // 小指以外は親指が担当
                        if col <= MAX_COL_CELLS / 2 {
                            Finger::LThumb
                        } else {
                            Finger::RThumb
                        }
                    }
                } else {
                    finger_from_x(col)
                };
                row_cells.push(Cell {
                    id: CellId::new(row, col),
                    finger,
                    occupied: false,
                });
            }
            cells.push(row_cells);
        }

        let mut geom = Geometry {
            name,
            cells,
            homes: HashMap::new(),
            key_placements: HashMap::new(),
        };

        // 固定文字（A..Z）を確保
        geom.reserve_letter_cells();
        // ホーム位置（ASDF / JKL;）
        geom.init_homes();

        Ok(geom)
    }

    /// Reserve letter blocks (using builder pattern)
    fn reserve_letter_cells(&mut self) {
        // row-idx [u], start-cell [cell], Vec of key names
        let positions = match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::get_letter_block_positions(),
            GeometryName::Ortho => OrthoBuilder::get_letter_block_positions(),
        };

        // 行ごとの処理
        for (row_idx, start_cell, names) in positions {
            self.reserve_row(row_idx, start_cell, names);
        }
    }

    /// 行ごとの処理
    fn reserve_row(&mut self, row_idx: usize, start_cell: usize, names: Vec<&'static str>) {
        // 1u key
        for (col_idx, name) in names.iter().enumerate() {
            // cell unit
            let col = start_cell + col_idx * U2CELL;
            let (x, y) = cell_to_key_center(row_idx, col, 1.0);

            self.key_placements.insert(
                name.to_string(),
                KeyPlacement {
                    placement_type: PlacementType::Fixed,
                    key_id: None, // アルファベットキーはKeyIdにないためNone
                    x,
                    y,
                    width_u: 1.0,
                    block_id: None, // 固定キーにはblockIdは不要
                },
            );

            // 1 x 1u (4 cell) を確保
            for i in 0..U2CELL {
                self.cells[row_idx][col + i].occupied = true;
            }
        }
    }

    /// Home positions (geometry-specific)
    fn init_homes(&mut self) {
        self.homes = match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::build_home_positions(),
            GeometryName::Ortho => OrthoBuilder::build_home_positions(),
        };
    }

    /// Calculate geometry-aware position for fixed key rectangles
    pub fn get_fixed_key_position(&self, row_idx: usize, col_idx: usize) -> (f32, f32) {
        match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::get_fixed_key_position(row_idx, col_idx),
            GeometryName::Ortho => OrthoBuilder::get_fixed_key_position(row_idx, col_idx),
        }
    }

    /// Calculate geometry-aware position for QWERTY labels
    pub fn get_qwerty_label_position(&self, row_idx: usize, char_idx: usize) -> (f32, f32) {
        match self.name {
            GeometryName::RowStagger => {
                RowStaggerBuilder::get_qwerty_label_position(row_idx, char_idx)
            }
            GeometryName::Ortho => OrthoBuilder::get_qwerty_label_position(row_idx, char_idx),
        }
    }
}

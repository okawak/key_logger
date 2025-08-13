use std::collections::HashMap;

use super::{
    builders::{GeometryBuilder, ortho::OrthoBuilder, row_stagger::RowStaggerBuilder},
    types::*,
    zoning::finger_from_x,
};
use crate::constants::{MAX_COL_CELLS, MAX_ROW_CELLS, U2CELL};
use crate::error::Result;

/// Geometry construction: 0.25u grid, fixed letters reservation, homes
/// 指ゾーンの最終決定は zoning::apply_zone_policy に委譲する
impl Geometry {
    pub fn build(name: GeometryName) -> Result<Self> {
        let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(MAX_ROW_CELLS);
        for row in 0..MAX_ROW_CELLS {
            let mut row_cells = Vec::with_capacity(MAX_COL_CELLS);
            for col in 0..MAX_COL_CELLS {
                let finger = if row == 0 {
                    if col as f32 <= MAX_COL_CELLS as f32 / 2.0 {
                        Finger::LThumb
                    } else {
                        Finger::RThumb
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
        };

        // 固定文字（A..Z）を確保
        geom.reserve_letter_blocks();
        // ホーム位置（ASDF / JKL;）
        geom.init_homes();

        Ok(geom)
    }

    /// Reserve letter blocks (using builder pattern)
    fn reserve_letter_blocks(&mut self) {
        let positions = match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::get_letter_block_positions(),
            GeometryName::Ortho => OrthoBuilder::get_letter_block_positions(),
        };

        for (row_idx, start_cell, count) in positions {
            self.reserve_run(row_idx, start_cell, count);
        }
    }
    fn reserve_run(&mut self, row_idx: usize, start_cell: usize, count: usize) {
        // 1u key
        for col in start_cell..(start_cell + count * U2CELL as usize) {
            for row in (row_idx * U2CELL as usize)..((row_idx + 1) * U2CELL as usize) {
                self.cells[row][col].occupied = true;
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

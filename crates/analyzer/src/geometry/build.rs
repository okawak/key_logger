use crate::{
    config::Config,
    constants::{COLUMN_STAGGER, MAX_COL_CELLS, ORTHO, ROW_STAGGER, U2CELL, cell_to_key_center},
    error::{KbOptError, Result},
    geometry::{
        builders::{GeometryBuilder, ortho::OrthoBuilder, row_stagger::RowStaggerBuilder},
        types::*,
        zoning::finger_from_x,
    },
    keys::str_to_keyid,
};

use std::collections::HashMap;

/// Geometry construction: 0.25u grid, fixed letters reservation, homes
impl Geometry {
    pub fn build(config: &Config) -> Result<Self> {
        let name = match config.solver.geometry.as_str() {
            ROW_STAGGER => GeometryName::RowStagger,
            ORTHO => GeometryName::Ortho,
            COLUMN_STAGGER => {
                return Err(KbOptError::Config(format!(
                    "{COLUMN_STAGGER} geometry is not yet implemented.",
                )));
            }
            _ => {
                unreachable!(); // validationで既にチェック済み
            }
        };
        let max_rows = config.solver.max_rows;
        let max_layers = match config.solver.version.as_str() {
            "v1" => 1, // v1はレイヤなし
            "v2" => config.v2.as_ref().unwrap().max_layers,
            "v3" => config.v3.as_ref().unwrap().max_layers,
            _ => unreachable!(), // validationで既にチェック済み
        };

        let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(max_rows);
        for row in 0..max_rows {
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
                        if col < MAX_COL_CELLS / 2 {
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
            max_layers,
        };

        // 固定文字（A..Z）を確保
        geom.reserve_cells(config);
        // ホーム位置（ASDF / JKL;）
        geom.init_homes(config);

        Ok(geom)
    }

    /// Reserve letter blocks (using builder pattern)
    fn reserve_cells(&mut self, config: &Config) {
        // row-idx [u], start-cell [cell], Vec of key names
        let positions = match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::get_fixed_key_positions(config),
            GeometryName::Ortho => OrthoBuilder::get_fixed_key_positions(config),
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
            let col = start_cell + col_idx * U2CELL; // 1u key
            let (x, y) = cell_to_key_center(row_idx, col, 1.0);

            self.key_placements.insert(
                name.to_string(),
                KeyPlacement {
                    placement_type: PlacementType::Fixed,
                    key_id: str_to_keyid(name),
                    x,
                    y,
                    width_u: 1.0,
                    layer: 0, // 固定キーはベースレイヤ
                },
            );

            // 1 x 1u (4 cell) を確保
            for i in 0..U2CELL {
                self.cells[row_idx][col + i].occupied = true;
            }
        }
    }

    /// Home positions (geometry-specific)
    fn init_homes(&mut self, config: &Config) {
        self.homes = match self.name {
            GeometryName::RowStagger => RowStaggerBuilder::build_home_positions(config),
            GeometryName::Ortho => OrthoBuilder::build_home_positions(config),
        };
    }
}

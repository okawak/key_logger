use crate::{
    config::Config,
    constants::{MIDDLE_CELL, cell_to_key_center},
    geometry::{
        builders::GeometryBuilder,
        types::{Finger, Finger::*},
    },
};
use std::collections::HashMap;

// Row offsets from middle cell (negative = left shift)
const ROW_OFFSETS: [i32; 4] = [-20, -20, -20, -20];

// Fixed key layout definition
const FIXED_KEYS: &[(usize, &[&str])] = &[
    (1, &["Z", "X", "C", "V", "B", "N", "M"]), // Bottom row: 7 keys
    (2, &["A", "S", "D", "F", "G", "H", "J", "K", "L"]), // Home row: 9 keys
    (3, &["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"]), // Top row: 10 keys
];

const DIGIT_KEYS: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

// Home row finger positions and their corresponding offsets
const HOME_FINGER_DATA: [(Finger, i32, usize); 10] = [
    (LPinky, -20, 2),  // A
    (LRing, -16, 2),   // S
    (LMiddle, -12, 2), // D
    (LIndex, -8, 2),   // F
    (LThumb, -8, 0),   // Same cell as LIndex but row 0
    (RIndex, 4, 2),    // J
    (RThumb, 4, 0),    // Same cell as RIndex but row 0
    (RMiddle, 8, 2),   // K
    (RRing, 12, 2),    // L
    (RPinky, 16, 2),   // ;
];

pub struct OrthoBuilder;

impl GeometryBuilder for OrthoBuilder {
    fn get_fixed_key_positions(config: &Config) -> Vec<(usize, usize, Vec<&'static str>)> {
        let mut positions = Vec::with_capacity(if config.solver.include_digits { 4 } else { 3 });

        // Add standard letter rows
        for &(row_idx, keys) in FIXED_KEYS {
            let start_cell = (MIDDLE_CELL as i32 + ROW_OFFSETS[row_idx - 1]) as usize;
            positions.push((row_idx, start_cell, keys.to_vec()));
        }

        // include_digitsがfalseの場合、数字行は固定とする
        if !config.solver.include_digits {
            let start_cell = (MIDDLE_CELL as i32 + ROW_OFFSETS[3]) as usize;
            positions.push((4, start_cell, DIGIT_KEYS.to_vec()));
        }

        positions
    }

    fn build_home_positions(_config: &Config) -> HashMap<Finger, (f32, f32)> {
        HOME_FINGER_DATA
            .iter()
            .map(|&(finger, offset, row)| {
                let cell = (MIDDLE_CELL as i32 + offset) as usize;
                (finger, cell_to_key_center(row, cell, 1.0))
            })
            .collect()
    }
}

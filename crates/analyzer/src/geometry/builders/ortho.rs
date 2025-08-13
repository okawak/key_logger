use super::super::types::*;
use super::GeometryBuilder;
use crate::constants::cell_to_key_center;
use std::collections::HashMap;

// Constants for cell calculations
const CELL_U: f32 = 0.25; // Each cell is 0.25u
const ONE_U: f32 = 1.0; // 1u in terms of cell units

pub struct OrthoBuilder;

impl GeometryBuilder for OrthoBuilder {
    fn get_letter_block_positions() -> Vec<(usize, usize, usize)> {
        vec![
            (1, 20, 7),  // Bottom row ZXCV: 7 keys
            (2, 20, 9),  // Middle row ASDF: 9 keys
            (3, 20, 10), // Top row QWERTY: 10 keys
        ]
    }

    fn build_home_positions() -> HashMap<Finger, (f32, f32)> {
        let mut homes = HashMap::new();

        // second row 2 -> 8 cell
        homes.insert(Finger::LPinky, cell_to_key_center(8, 20, 1)); // A
        homes.insert(Finger::LRing, cell_to_key_center(8, 24, 1)); // S
        homes.insert(Finger::LMiddle, cell_to_key_center(8, 28, 1)); // D
        homes.insert(Finger::LIndex, cell_to_key_center(8, 32, 1)); // F
        homes.insert(Finger::LThumb, cell_to_key_center(0, 32, 1));
        homes.insert(Finger::RIndex, cell_to_key_center(8, 44, 1)); // J
        homes.insert(Finger::RMiddle, cell_to_key_center(8, 48, 1)); // K
        homes.insert(Finger::RRing, cell_to_key_center(8, 52, 1)); // L
        homes.insert(Finger::RPinky, cell_to_key_center(8, 56, 1)); // ;
        homes.insert(Finger::RThumb, cell_to_key_center(0, 56, 1));

        homes
    }

    fn get_fixed_key_position(row_idx: usize, col_idx: usize) -> (f32, f32) {
        // Orthoの場合は行オフセットを無視
        let x0 = col_idx as f32 * CELL_U;
        let y0 = row_idx as f32 - 0.5;
        (x0, y0)
    }

    fn get_qwerty_label_position(row_idx: usize, char_idx: usize) -> (f32, f32) {
        let x = (char_idx as f32 + 0.5) * ONE_U;
        let y = row_idx as f32;
        (x, y)
    }
}

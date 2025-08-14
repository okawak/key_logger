use super::super::types::*;
use super::GeometryBuilder;
use crate::constants::U2CELL;
use crate::constants::cell_to_key_center;
use std::collections::HashMap;

pub struct OrthoBuilder;

impl GeometryBuilder for OrthoBuilder {
    /// row-idx [u], start-cell [cell], Vec of key names
    fn get_letter_block_positions() -> Vec<(usize, usize, Vec<&'static str>)> {
        vec![
            (1, 20, vec!["Z", "X", "C", "V", "B", "N", "M"]), // Bottom row ZXCV: 7 keys
            (2, 20, vec!["A", "S", "D", "F", "G", "H", "J", "K", "L"]), // Middle row ASDF: 9 keys
            (
                3,
                20,
                vec!["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
            ), // Top row QWERTY: 10 keys
        ]
    }

    fn build_home_positions() -> HashMap<Finger, (f32, f32)> {
        let mut homes = HashMap::new();

        // second row 2 -> 8 cell
        homes.insert(Finger::LPinky, cell_to_key_center(8, 20, 1.0)); // A
        homes.insert(Finger::LRing, cell_to_key_center(8, 24, 1.0)); // S
        homes.insert(Finger::LMiddle, cell_to_key_center(8, 28, 1.0)); // D
        homes.insert(Finger::LIndex, cell_to_key_center(8, 32, 1.0)); // F
        homes.insert(Finger::LThumb, cell_to_key_center(0, 32, 1.0));
        homes.insert(Finger::RIndex, cell_to_key_center(8, 44, 1.0)); // J
        homes.insert(Finger::RMiddle, cell_to_key_center(8, 48, 1.0)); // K
        homes.insert(Finger::RRing, cell_to_key_center(8, 52, 1.0)); // L
        homes.insert(Finger::RPinky, cell_to_key_center(8, 56, 1.0)); // ;
        homes.insert(Finger::RThumb, cell_to_key_center(0, 56, 1.0));

        homes
    }

    fn get_fixed_key_position(row_idx: usize, col_idx: usize) -> (f32, f32) {
        // Orthoの場合は行オフセットを無視
        let x0 = col_idx as f32 / U2CELL as f32;
        let y0 = row_idx as f32 - 0.5;
        (x0, y0)
    }

    fn get_qwerty_label_position(row_idx: usize, char_idx: usize) -> (f32, f32) {
        let x = char_idx as f32 + 0.5;
        let y = row_idx as f32;
        (x, y)
    }
}

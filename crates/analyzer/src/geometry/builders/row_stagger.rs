use super::super::types::*;
use super::GeometryBuilder;
use crate::constants::{cell_to_key_center, U2CELL};
use std::collections::HashMap;

// Constants for cell calculations
const ONE_U: f32 = 1.0; // 1u in terms of cell units

pub struct RowStaggerBuilder;

impl GeometryBuilder for RowStaggerBuilder {
    /// row-idx [u], start-cell [cell], vec of key names
    fn get_letter_block_positions() -> Vec<(usize, usize, Vec<&'static str>)> {
        vec![
            (1, 22, vec!["Z", "X", "C", "V", "B", "N", "M"]), // Bottom row ZXCV: 7 keys
            (2, 20, vec!["A", "S", "D", "F", "G", "H", "J", "K", "L"]), // Middle row ASDF: 9 keys
            (
                3,
                19,
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
        // アルファベット文字の固定キー位置を計算（col_idxは文字インデックス0,1,2...）
        let alphabet_start_positions = [0.0, 2.25, 1.75, 1.50, 0.0]; // [row0(親指), row1(ZXCV), row2(ASDF), row3(QWERTY), row4(数字)]
        let alphabet_start_u = alphabet_start_positions[row_idx];

        // セル単位でのアルファベット開始位置計算
        let alphabet_start_cells = (alphabet_start_u * U2CELL as f32).round() as usize;

        // col_idx番目のキーのセル開始位置（1u = 4セル）
        let key_start_cells = alphabet_start_cells + col_idx * U2CELL;

        // 固定キー枠の位置（左端）を計算（簡略化）
        let x0 = key_start_cells as f32 * 1.0 / U2CELL as f32;
        let y0 = row_idx as f32 - 0.5;
        (x0, y0)
    }

    fn get_qwerty_label_position(row_idx: usize, char_idx: usize) -> (f32, f32) {
        // アルファベット開始位置（絶対座標u単位）
        let alphabet_start_positions = [0.0, 2.25, 1.75, 1.50, 0.0]; // [row0(親指), row1(ZXCV), row2(ASDF), row3(QWERTY), row4(数字)]
        let alphabet_start_u = alphabet_start_positions[row_idx];

        // セル単位でのアルファベット開始位置計算
        let alphabet_start_cells = (alphabet_start_u * U2CELL as f32).round() as usize;

        // char_idx番目のキーのセル開始位置（1u = 4セル）
        let key_start_cells = alphabet_start_cells + char_idx * U2CELL;

        // 固定キー枠の位置（左端）を計算 - get_fixed_key_positionと同じロジック
        let key_left_u = key_start_cells as f32 * 1.0 / U2CELL as f32;

        // ラベル位置はキーの中心
        let key_center_u = key_left_u + ONE_U / 2.0;
        let y = row_idx as f32;

        (key_center_u, y)
    }
}

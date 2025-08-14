use super::types::*;

pub mod ortho;
pub mod row_stagger;

use std::collections::HashMap;

/// 共通のジオメトリビルダートレイト
pub trait GeometryBuilder {
    /// 固定文字ブロックの位置を定義
    /// (下からの行インデックス, 左から始まりのcell, キーの数)
    /// row-idx [u], start-cell [cell], Vec of key names
    fn get_letter_block_positions() -> Vec<(usize, usize, Vec<&'static str>)>; // (row_idx, start_u, key_name)

    /// ジオメトリ固有のホームポジション全体を設定
    fn build_home_positions() -> HashMap<Finger, (f32, f32)>;

    /// 固定キー矩形の位置を計算
    fn get_fixed_key_position(row_idx: usize, col_idx: usize) -> (f32, f32);

    /// QWERTYラベルの位置を計算
    fn get_qwerty_label_position(row_idx: usize, char_idx: usize) -> (f32, f32);
}

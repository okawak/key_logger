use super::types::*;

pub mod ortho;
pub mod row_stagger;

use std::collections::HashMap;

/// 共通のジオメトリビルダートレイト
pub trait GeometryBuilder {
    /// 固定文字ブロックの位置を定義
    /// (下からの行インデックス, 左から始まりのcell, キーの数)
    /// row-idx \[u\], start-cell \[cell\], Vec of key names
    fn get_letter_block_positions() -> Vec<(usize, usize, Vec<&'static str>)>; // (row_idx, start_u, key_name)

    /// ジオメトリ固有のホームポジション全体を設定
    fn build_home_positions() -> HashMap<Finger, (f32, f32)>;
}

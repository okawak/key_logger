pub mod custom;
pub mod ortho;
pub mod row_stagger;

use crate::{config::Config, geometry::types::*};

use std::collections::HashMap;

/// 共通のジオメトリビルダートレイト
pub trait GeometryBuilder {
    /// 固定文字ブロックの位置を定義
    /// (下からの行インデックス, 左から始まりのcell, キーの数)
    /// row-idx \[u\], start-cell \[cell\], Vec of key names
    fn get_fixed_key_positions(config: &Config) -> Vec<(usize, usize, Vec<&'static str>)>; // (row_idx, start_u, key_name)

    /// ジオメトリ固有のホームポジション全体を設定
    fn build_home_positions(config: &Config) -> HashMap<Finger, (f64, f64)>;
}

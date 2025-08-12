use super::types::*;

pub mod col_stagger;
pub mod ortho;
pub mod row_stagger;

/// 共通のジオメトリビルダートレイト
pub trait GeometryBuilder {
    /// 行仕様を生成
    fn build_rows(cells_per_row: usize) -> Vec<RowSpec>;

    /// 列オフセット（ColStaggerのみ使用）
    fn build_col_stagger_y(cells_per_row: usize) -> Vec<f32>;

    /// 固定文字ブロックの位置を定義
    fn get_letter_block_positions() -> Vec<(usize, f32, usize)>; // (row_idx, start_u, count_1u)

    /// ホーム位置計算用のヘルパー
    fn calculate_home_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        char_idx: usize,
    ) -> (f32, f32);

    /// 固定キー矩形の位置を計算
    fn get_fixed_key_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        col_idx: usize,
    ) -> (f32, f32);

    /// QWERTYラベルの位置を計算
    fn get_qwerty_label_position(
        geometry_cfg: &GeometryConfig,
        row_idx: usize,
        char_idx: usize,
    ) -> (f32, f32);

    /// ジオメトリ固有のホームポジション全体を設定
    fn build_home_positions(
        geometry_cfg: &GeometryConfig,
    ) -> std::collections::HashMap<Finger, (f32, f32)>;
}

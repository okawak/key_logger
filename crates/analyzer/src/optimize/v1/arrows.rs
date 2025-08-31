use crate::{constants::U2CELL, geometry::Geometry, keys::ArrowKey};

/// 矢印キー配置パターン
#[derive(Debug, Clone)]
pub enum ArrowPlacement {
    /// 横一列配置
    /// 配置順序: ← ↓ ↑ → （位置 i, i+s₀, i+2s₀, i+3s₀）
    Horizontal { r: usize, i: usize },
    /// T字型配置
    /// 下段: ← ↓ → (行r, 位置i, i+s₀, i+2s₀)
    /// 上段: ↑ (行r+1, 位置i+s₀)
    TShape { r: usize, i: usize },
}

/// 横一列配置候補集合Ω_H
/// Ω_H = {(r, i) | r ∈ R, 0 ≤ i ≤ C - 4s₀, 連続空間確保}
pub fn generate_horizontal_candidates(geom: &Geometry) -> Vec<ArrowPlacement> {
    let mut candidates = Vec::new();
    let s0 = U2CELL; // 基本セルサイズ s₀ = 4

    // r ∈ R (行の集合)
    for r in 0..geom.cells.len() {
        let row_len = geom.cells[r].len();

        // 0 ≤ i ≤ C - 4s₀
        if row_len >= 4 * s0 {
            for i in 0..=(row_len - 4 * s0) {
                // 連続空間確保条件: O_{rj} = 0 ∀j ∈ [i, i + 4s₀ - 1]
                let space_available = (i..i + 4 * s0).all(|j| !geom.cells[r][j].occupied);

                if space_available {
                    candidates.push(ArrowPlacement::Horizontal { r, i });
                }
            }
        }
    }

    candidates
}

/// T字型配置候補集合Ω_T
/// Ω_T = {(r, i) | r + 1 ≤ R_max, 0 ≤ i ≤ C - 3s₀, T字空間確保}
pub fn generate_t_shape_candidates(geom: &Geometry) -> Vec<ArrowPlacement> {
    let mut candidates = Vec::new();
    let s0 = U2CELL; // 基本セルサイズ s₀ = 4

    // r + 1 ≤ R_max
    for r in 0..geom.cells.len() {
        if r + 1 >= geom.cells.len() {
            continue;
        }

        let bottom_row_len = geom.cells[r].len();
        let top_row_len = geom.cells[r + 1].len();

        // 0 ≤ i ≤ C - 3s₀
        if bottom_row_len >= 3 * s0 && top_row_len >= 2 * s0 {
            let max_i = (bottom_row_len - 3 * s0).min(top_row_len - 2 * s0);

            for i in 0..=max_i {
                // T字空間確保条件:
                // 下段（行r）: O_{rj} = 0 ∀j ∈ [i, i + 3s₀ - 1]
                let bottom_space =
                    (i..i + 3 * s0).all(|j| j < geom.cells[r].len() && !geom.cells[r][j].occupied);

                // 上段（行r+1）: O_{(r+1)j} = 0 ∀j ∈ [i+s₀, i+2s₀-1]
                let top_space = (i + s0..i + 2 * s0)
                    .all(|j| j < geom.cells[r + 1].len() && !geom.cells[r + 1][j].occupied);

                if bottom_space && top_space {
                    candidates.push(ArrowPlacement::TShape { r, i });
                }
            }
        }
    }

    candidates
}

/// docs/v1.md Section 5.3に対応: 矢印キーの配置順序を取得
impl ArrowPlacement {
    /// 配置される矢印キーとその位置を取得
    /// 戻り値: Vec<(ArrowKey, row, col)>
    pub fn get_arrow_positions(&self) -> Vec<(ArrowKey, usize, usize)> {
        let s0 = U2CELL;

        match self {
            // docs/v1.md Section 3.2.1: 配置順序 ← ↓ ↑ →
            ArrowPlacement::Horizontal { r, i } => vec![
                (ArrowKey::Left, *r, *i),           // 位置 i
                (ArrowKey::Down, *r, *i + s0),      // 位置 i + s₀
                (ArrowKey::Up, *r, *i + 2 * s0),    // 位置 i + 2s₀
                (ArrowKey::Right, *r, *i + 3 * s0), // 位置 i + 3s₀
            ],

            // docs/v1.md Section 3.2.2: T字配置
            ArrowPlacement::TShape { r, i } => vec![
                (ArrowKey::Left, *r, *i),           // 下段: 位置 i
                (ArrowKey::Down, *r, *i + s0),      // 下段: 位置 i + s₀
                (ArrowKey::Right, *r, *i + 2 * s0), // 下段: 位置 i + 2s₀
                (ArrowKey::Up, *r + 1, *i + s0),    // 上段: 位置 i + s₀
            ],
        }
    }

    /// 占有されるセル(r, i)の一覧を取得（物理非重複制約用）
    pub fn get_occupied_cells(&self) -> Vec<(usize, usize)> {
        let s0 = U2CELL;
        let mut cells = Vec::new();

        match self {
            ArrowPlacement::Horizontal { r, i } => {
                // 4s₀個のセルを占有
                for j in *i..*i + 4 * s0 {
                    cells.push((*r, j));
                }
            }
            ArrowPlacement::TShape { r, i } => {
                // 下段: 3s₀個のセル
                for j in *i..*i + 3 * s0 {
                    cells.push((*r, j));
                }
                // 上段: s₀個のセル
                for j in *i + s0..*i + 2 * s0 {
                    cells.push((*r + 1, j));
                }
            }
        }

        cells
    }
}

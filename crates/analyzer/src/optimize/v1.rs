use std::collections::{BTreeSet, HashMap};

use crate::constants::U2MM;
use crate::geometry::{
    Geometry,
    fitts::euclid_u,
    precompute::{Precompute, compute_free_blocks},
    types::{BlockId, CELL_U, CellId, KeyCandidates, ONE_U, cells_from_u},
};
use crate::keys::{ArrowKey, KeyId};

/// 矢印キー定数
pub const ARROW_KEYS: [KeyId; 4] = [
    KeyId::Arrow(ArrowKey::Up),
    KeyId::Arrow(ArrowKey::Down),
    KeyId::Arrow(ArrowKey::Left),
    KeyId::Arrow(ArrowKey::Right),
];

/// キー種別判定
pub fn is_arrow(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Arrow(_))
}

pub fn is_digit_or_f(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Digit(_) | KeyId::Function(_))
}

/// 幅候補（0.25u 刻み）。数字/F/矢印は 1u 固定。
pub fn width_candidates_for_key(key_id: &KeyId) -> Vec<f32> {
    if is_arrow(key_id) || is_digit_or_f(key_id) {
        vec![1.0]
    } else {
        // 0.25u 刻みで 1.00..2.00 あたり（最小幅1uを保証）
        let mut v = Vec::new();
        let mut w = 1.00f32;
        while w <= 2.00 + 1e-6 {
            v.push((w * 100.0).round() / 100.0);
            w += 0.25;
        }
        v
    }
}

/// 配置候補（通常キー）
#[derive(Debug, Clone)]
pub struct Cand {
    pub key: KeyId,
    pub row: usize,
    pub start_col: usize, // 0.25u index
    pub w_u: f32,
    pub cost_ms: f64, // f_k を掛ける前の素コスト
    pub cover_cells: Vec<CellId>,
}

// BlockIdはgeometry/types.rsに移動しました

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub center: (f32, f32),       // [u]
    pub cover_cells: [CellId; 4], // この1uが覆う 0.25u セル
}

/// v1: 全キーが全行に配置可能な候補を生成
pub fn generate_v1_key_candidates(
    geom: &Geometry,
    movable_keys: &[KeyId],
) -> HashMap<KeyId, KeyCandidates> {
    let free_blocks = compute_free_blocks(geom);
    let mut out = HashMap::new();

    for &k in movable_keys {
        let widths = width_candidates_for_key(&k);
        let mut starts = Vec::new();

        // 全行に配置可能
        for r in 0..geom.cells.len() {
            if r >= free_blocks.len() {
                continue;
            }
            for &(start, len) in &free_blocks[r] {
                for i in start..(start + len) {
                    let mut fits = Vec::new();
                    for &w in &widths {
                        let need = cells_from_u(w);
                        if i + need <= start + len {
                            fits.push(w);
                        }
                    }
                    if !fits.is_empty() {
                        starts.push((CellId::new(r, i), fits));
                    }
                }
            }
        }
        out.insert(k, KeyCandidates { starts });
    }
    out
}

/// v1: 全空きセルを矢印キー配置候補とする
pub fn generate_v1_arrow_region(geom: &Geometry) -> (Vec<CellId>, Vec<(CellId, CellId)>) {
    let mut arrow_cells = Vec::new();

    // 全ての空きセルを矢印キー候補に追加
    for r in 0..geom.cells.len() {
        for c in 0..geom.cells[r].len() {
            if !geom.cells[r][c].occupied {
                arrow_cells.push(CellId::new(r, c));
            }
        }
    }

    // 4近傍隣接エッジを生成
    let arrow_set: std::collections::HashSet<_> = arrow_cells.iter().cloned().collect();
    let mut arrow_edges = Vec::new();

    for &cell_id in &arrow_cells {
        let (r, c) = (cell_id.row, cell_id.col);
        let neighbors = [
            (r, c.wrapping_add(1)),
            (r, c.wrapping_sub(1)),
            (r + 1, c),
            (r.wrapping_sub(1), c),
        ];

        for (rr, cc) in neighbors {
            if rr < geom.cells.len() && cc < geom.cells[rr].len() {
                let neighbor_id = CellId::new(rr, cc);
                if arrow_set.contains(&neighbor_id) {
                    arrow_edges.push((cell_id, neighbor_id));
                }
            }
        }
    }

    (arrow_cells, arrow_edges)
}

/// Precomputeから通常キーの候補を生成
pub fn build_candidates_from_precompute(
    geom: &Geometry,
    movable: &BTreeSet<KeyId>,
    precompute: &Precompute,
    opt: &super::SolveOptions,
) -> Vec<Cand> {
    let mut out = Vec::new();

    for &key in movable {
        if let Some(key_candidates) = precompute.key_cands.get(&key) {
            for (start_cell, widths) in &key_candidates.starts {
                for &w_u in widths {
                    let w_cells = cells_from_u(w_u);
                    if w_cells == 0 {
                        continue;
                    }

                    // 中心セルの指でホームを取る（簡略化）
                    let c_center = start_cell.col + w_cells / 2;
                    let cx = start_cell.col as f32 * CELL_U + w_u * 0.5;
                    let cy = start_cell.row as f32;

                    let finger = geom.cells[start_cell.row][c_center].finger;
                    let home = geom.homes.get(&finger).cloned().unwrap_or((cx, cy));
                    let d_mm = (euclid_u((cx, cy), home) as f64) * U2MM;
                    let w_mm = (w_u as f64) * U2MM;
                    let t_ms = opt.a_ms + opt.b_ms * ((d_mm / w_mm + 1.0).log2());

                    let cover_cells: Vec<CellId> = (start_cell.col..start_cell.col + w_cells)
                        .map(|cc| CellId::new(start_cell.row, cc))
                        .collect();

                    out.push(Cand {
                        key,
                        row: start_cell.row,
                        start_col: start_cell.col,
                        w_u,
                        cost_ms: t_ms,
                        cover_cells,
                    });
                }
            }
        }
    }
    out
}

/// Precomputeから矢印用ブロックを生成
pub fn build_blocks_from_precompute(
    _geom: &Geometry,
    precompute: &Precompute,
) -> (Vec<Block>, HashMap<BlockId, usize>) {
    let mut blocks = Vec::new();
    let mut index = HashMap::new();

    // 1uブロック単位でグループ化
    let mut block_cells: HashMap<(usize, usize), Vec<CellId>> = HashMap::new();

    for &cell_id in &precompute.arrow_cells {
        let row = cell_id.row;
        let bcol = cell_id.col / cells_from_u(ONE_U);
        block_cells.entry((row, bcol)).or_default().push(cell_id);
    }

    for ((row, bcol), cells) in block_cells {
        if cells.len() == cells_from_u(ONE_U) {
            // 完全な1uブロックのみ追加（簡略化）
            let start_col = bcol * cells_from_u(ONE_U);
            let x0 = start_col as f32 * CELL_U;
            let cx = x0 + 0.5 * ONE_U;
            let cy = row as f32;

            let cover_cells = [
                CellId::new(row, start_col),
                CellId::new(row, start_col + 1),
                CellId::new(row, start_col + 2),
                CellId::new(row, start_col + 3),
            ];

            let block_id = BlockId::new(row, bcol);
            let idx = blocks.len();

            blocks.push(Block {
                id: block_id,
                center: (cx, cy),
                cover_cells,
            });
            index.insert(block_id, idx);
        }
    }

    (blocks, index)
}

/// Precomputeから隣接エッジを生成
pub fn build_adjacency_from_precompute(
    blocks: &[Block],
    precompute: &Precompute,
) -> Vec<(usize, usize)> {
    let mut block_index: HashMap<CellId, usize> = HashMap::new();
    for (i, block) in blocks.iter().enumerate() {
        for &cell_id in &block.cover_cells {
            block_index.insert(cell_id, i);
        }
    }

    let mut edges = Vec::new();
    for &(from_cell, to_cell) in &precompute.arrow_edges {
        if let (Some(&from_block), Some(&to_block)) =
            (block_index.get(&from_cell), block_index.get(&to_cell))
            && from_block != to_block
        {
            edges.push((from_block, to_block));
        }
    }

    edges
}

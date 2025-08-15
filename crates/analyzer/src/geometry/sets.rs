use std::collections::HashMap;

use super::types::*;
use crate::keys::KeyId;

/// 最適化問題の集合（キー候補、矢印候補等）
#[derive(Debug, Clone)]
pub struct OptimizationSets {
    pub key_cands: HashMap<KeyId, KeyCandidates>, // General key candidates \mathcal{I}^g_k
    pub arrow_cells: Vec<CellId>,                 // Arrow allowed cells \mathcal{A}^g
    pub arrow_edges: Vec<(CellId, CellId)>,       // 4-neighborhood directed edge set E_g
}

/// 各行の連続する空きセル区間（固定文字以外のセル範囲）を抽出
pub fn extract_free_cell_intervals(geom: &Geometry) -> Vec<Vec<(usize, usize)>> {
    let mut out = vec![vec![]; geom.cells.len()];
    for (r, row_blocks) in out.iter_mut().enumerate().take(geom.cells.len()) {
        let mut c = 0usize;
        while c < geom.cells[r].len() {
            while c < geom.cells[r].len() && geom.cells[r][c].occupied {
                c += 1;
            }
            let start = c;
            while c < geom.cells[r].len() && !geom.cells[r][c].occupied {
                c += 1;
            }
            let len = c.saturating_sub(start);
            if len > 0 {
                row_blocks.push((start, len));
            }
        }
    }
    out
}

impl Geometry {}

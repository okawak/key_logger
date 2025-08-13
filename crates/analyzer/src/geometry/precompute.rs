use std::collections::{HashMap, HashSet};

use super::types::*;
use crate::keys::{KeyId, allowed_widths};

/// Preprocessing output (bundle passed to optimization)
#[derive(Debug, Clone)]
pub struct Precompute {
    pub key_cands: HashMap<KeyId, KeyCandidates>, // General key candidates \mathcal{I}^g_k
    pub arrow_cells: Vec<CellId>,                 // Arrow allowed cells \mathcal{A}^g
    pub arrow_edges: Vec<(CellId, CellId)>,       // 4-neighborhood directed edge set E_g
}

/// Extract "consecutive free blocks" (cell intervals other than fixed characters) for each row
pub fn compute_free_blocks(geom: &Geometry) -> Vec<Vec<(usize, usize)>> {
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

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

impl Geometry {
    /// Extract "consecutive free blocks" (cell intervals other than fixed characters) for each row
    fn compute_free_blocks(&self) -> Vec<Vec<(usize, usize)>> {
        let mut out = vec![vec![]; self.cells.len()];
        for (r, row_blocks) in out.iter_mut().enumerate().take(self.cells.len()) {
            let mut c = 0usize;
            while c < self.cells[r].len() {
                while c < self.cells[r].len() && self.cells[r][c].occupied {
                    c += 1;
                }
                let start = c;
                while c < self.cells[r].len() && !self.cells[r][c].occupied {
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
}

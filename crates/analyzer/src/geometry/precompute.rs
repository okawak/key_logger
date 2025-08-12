use std::collections::{HashMap, HashSet};

use super::policy::Policy;
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
        let mut out = vec![vec![]; self.cfg.rows.len()];
        for (r, row_blocks) in out.iter_mut().enumerate().take(self.cfg.rows.len()) {
            let mut c = 0usize;
            while c < self.cells_per_row {
                while c < self.cells_per_row && self.cells[r][c].fixed_occupied {
                    c += 1;
                }
                let start = c;
                while c < self.cells_per_row && !self.cells[r][c].fixed_occupied {
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

    /// Generate general key candidates \mathcal{I}^g_k (row crossing prohibited)
    pub fn generate_key_candidates(
        &self,
        movable_keys: &[KeyId],
        policy: &Policy,
    ) -> HashMap<KeyId, KeyCandidates> {
        let free_blocks = self.compute_free_blocks();
        let mut out = HashMap::new();

        for &k in movable_keys {
            let widths = allowed_widths(&k);
            let allow_rows = policy
                .allowed_rows
                .get(&k)
                .cloned()
                .unwrap_or_else(|| (0..self.cfg.rows.len()).collect());

            let mut starts = Vec::new();
            for r in allow_rows {
                if r >= free_blocks.len() {
                    continue;
                }
                for &(start, len) in &free_blocks[r] {
                    for i in start..(start + len) {
                        let mut fits = Vec::new();
                        for &w in widths {
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

    /// Arrow allowed cell set \mathcal{A}^g and 4-neighborhood directed edge set E_g (for connected flow)
    pub fn build_arrow_region_and_edges(
        &self,
        policy: &Policy,
    ) -> (Vec<CellId>, Vec<(CellId, CellId)>) {
        let mut allow: HashSet<CellId> = HashSet::new();
        for band in &policy.arrow_bands {
            let r = band.row;
            if r >= self.cfg.rows.len() {
                continue;
            }
            let rowspec = &self.cfg.rows[r];

            // [x0,x1) → cell range (half-open interval)
            let x0 = (band.x0_u - rowspec.offset_u) / CELL_U - 0.5;
            let x1 = (band.x1_u - rowspec.offset_u) / CELL_U - 0.5;
            let c0 = x0.ceil().clamp(0.0, rowspec.cells as f32) as usize;
            let c1 = x1.floor().clamp(0.0, rowspec.cells as f32) as usize;

            for c in c0..c1 {
                if !self.cells[r][c].fixed_occupied {
                    allow.insert(CellId::new(r, c));
                }
            }
        }

        let mut allow_vec: Vec<CellId> = allow.iter().cloned().collect();
        allow_vec.sort_by_key(|cid| (cid.row, cid.col));

        // 4-neighborhood directed edges
        let allow_set: HashSet<_> = allow_vec.iter().cloned().collect();
        let mut edges = Vec::new();
        for cid in &allow_vec {
            let (r, c) = (cid.row, cid.col);
            let neigh = [
                (r, c.wrapping_add(1)),
                (r, c.wrapping_sub(1)),
                (r + 1, c),
                (r.wrapping_sub(1), c),
            ];
            for (rr, cc) in neigh {
                if rr < self.cfg.rows.len() && cc < self.cells_per_row {
                    let nid = CellId::new(rr, cc);
                    if allow_set.contains(&nid) {
                        edges.push((*cid, nid));
                    }
                }
            }
        }
        (allow_vec, edges)
    }
}

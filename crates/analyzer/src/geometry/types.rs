use std::collections::HashMap;

/// Keyboard array type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryName {
    RowStagger,
    ColStagger,
    Ortho,
}

/// Finger type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Finger {
    LPinky,
    LRing,
    LMiddle,
    LIndex,
    LThumb,
    RThumb,
    RIndex,
    RMiddle,
    RRing,
    RPinky,
}

/// Cell ID (0.25u)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellId {
    pub row: usize,
    pub col: usize,
}

impl CellId {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// Information for a single cell
#[derive(Debug, Clone)]
pub struct Cell {
    pub id: CellId,
    pub finger: Finger,
}

/// Overall geometry
#[derive(Debug, Clone)]
pub struct Geometry {
    pub name: GeometryName,
    pub cells: Vec<Vec<Cell>>,          // cells[row][col]
    pub homes: HashMap<Finger, CellId>, // Finger → home coordinates [u]
}

/// Candidate set for general keys (start cell and allowed widths)
#[derive(Debug, Clone)]
pub struct KeyCandidates {
    pub starts: Vec<(CellId, Vec<f32>)>, // (Start cell, width candidates)
}

pub(crate) const ONE_U: f32 = 1.0;
pub(crate) const CELL_U: f32 = 0.25;

#[inline]
pub(crate) fn cells_from_u(u: f32) -> usize {
    ((u / CELL_U).round() as i32).max(0) as usize
}
#[inline]
#[allow(dead_code)]
pub(crate) fn u_from_cells(c: usize) -> f32 {
    c as f32 * CELL_U
}

#[inline]
#[allow(dead_code)]
pub(crate) fn row_idx(r: usize) -> usize {
    r
}

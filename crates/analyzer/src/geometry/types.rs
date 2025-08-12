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

/// Row specification
#[derive(Debug, Clone)]
pub struct RowSpec {
    pub offset_u: f32, // Left edge of the row (x-axis) [u]
    pub base_y_u: f32, // Base line of the row (y-axis) [u]
    pub width_u: f32,  // Total width [u]
    pub cells: usize,  // Number of cells in the row (width/0.25)
}

#[derive(Debug, Clone)]
pub struct GeometryConfig {
    pub cell_pitch_u: f32,             // 0.25u
    pub rows: Vec<RowSpec>,            // Row specifications
    pub col_stagger_y: Vec<f32>,       // Additional y offset per column for col-stagger
    pub finger_x_boundaries: [f32; 9], // Finger zone boundaries (ascending order)
    pub thumb_row: usize,              // Thumb row index (e.g., 4)
}

/// Information for a single cell
#[derive(Debug, Clone)]
pub struct Cell {
    pub id: CellId,
    pub center_x_u: f32,
    pub center_y_u: f32,
    pub finger: Finger,
    pub fixed_occupied: bool, // Fixed occupied by characters, etc.
}

/// Overall geometry
#[derive(Debug, Clone)]
pub struct Geometry {
    pub name: GeometryName,
    pub cfg: GeometryConfig,
    pub cells: Vec<Vec<Cell>>,              // cells[row][col]
    pub homes: HashMap<Finger, (f32, f32)>, // Finger → home coordinates [u]
    pub cells_per_row: usize,
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
pub(crate) fn finger_from_x(x: f32, b: &[f32; 9]) -> Finger {
    // Assumes b: [3, 5, 7, 8, 9, 11, 13, 15, 15]
    use Finger::*;
    if x < b[0] {
        return LPinky;
    }
    if x < b[1] {
        return LRing;
    }
    if x < b[2] {
        return LMiddle;
    }
    if x < b[3] {
        return LIndex;
    }
    if x < b[4] {
        return RIndex;
    }
    if x < b[5] {
        return RMiddle;
    }
    if x < b[6] {
        return RRing;
    }
    RPinky
}

#[inline]
#[allow(dead_code)]
pub(crate) fn row_idx(r: usize) -> usize {
    r
}

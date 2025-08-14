use crate::keys::KeyId;
use std::collections::HashMap;

/// Keyboard layout type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryName {
    RowStagger,
    Ortho,
    // other layout is under development
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

/// 1u ブロック（矢印用の占有単位）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId {
    pub row: usize,
    pub bcol: usize, // 1u ブロック列（0.25u 4セルごと）
}

impl BlockId {
    pub fn new(row: usize, bcol: usize) -> Self {
        Self { row, bcol }
    }
}

/// Information for a single cell
#[derive(Debug, Clone)]
pub struct Cell {
    pub id: CellId,
    pub finger: Finger,
    pub occupied: bool,
}

/// キー配置タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementType {
    Fixed,     // 固定キー（アルファベットなど）
    Optimized, // 最適化されたキー
    Arrow,     // 矢印キー
}

/// キー配置情報（通常キーと矢印キー統一）
#[derive(Debug, Clone)]
pub struct KeyPlacement {
    pub key_name: String,      // キー名（文字列）
    pub key_id: Option<KeyId>, // 定義されたキーID（オプション）
    pub row: usize,
    pub start_col: usize, // 0.25u index
    pub width_u: f32,
    pub placement_type: PlacementType,
    pub block_id: Option<BlockId>, // 矢印キー用のブロックID（オプション）
}

/// Overall geometry
#[derive(Debug, Clone)]
pub struct Geometry {
    pub name: GeometryName,
    pub cells: Vec<Vec<Cell>>,              // cells[row][col]
    pub homes: HashMap<Finger, (f32, f32)>, // Finger → home coordinates [R^2]
    pub key_placements: Vec<KeyPlacement>,  // store all key placements
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

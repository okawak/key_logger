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
    pub row_u: usize, // 1u ブロック行（u単位）
    pub col_u: usize, // 1u ブロック列（0.25u 4セルごと）
}

impl BlockId {
    pub fn new(row_u: usize, col_u: usize) -> Self {
        Self { row_u, col_u }
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
    pub placement_type: PlacementType,
    pub key_id: Option<KeyId>,        // 定義されたキーID（オプション）
    pub x: f32,                       // x coordinate [mm]
    pub y: f32,                       // y coordinate [mm]
    pub width_u: f32,                 // キー幅 [u]
    pub block_id: Option<BlockId>,    // 矢印キー用のブロックID（オプション）
    pub layer: u8,                    // レイヤ番号（0=ベースレイヤ、1以上=モディファイアレイヤ）
    pub modifier_key: Option<String>, // レイヤの場合のモディファイアキー名
}

/// Overall geometry
#[derive(Debug, Clone)]
pub struct Geometry {
    pub name: GeometryName,
    pub cells: Vec<Vec<Cell>>,                         // cells[row][col]
    pub homes: HashMap<Finger, (f32, f32)>,            // Finger → home coordinates [R^2]
    pub key_placements: HashMap<String, KeyPlacement>, // store all key placements
    pub max_layer: u8,                                 // 最大レイヤ番号（最適化結果に応じて変動）
}

/// Candidate set for general keys (start cell and allowed widths)
#[derive(Debug, Clone)]
pub struct KeyCandidates {
    pub starts: Vec<(CellId, Vec<f32>)>, // (Start cell, width candidates)
}

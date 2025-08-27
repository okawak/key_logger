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
    Digit,     // 数字キー
}

/// キー配置情報（通常キーと矢印キー統一）
#[derive(Debug, Clone)]
pub struct KeyPlacement {
    pub placement_type: PlacementType,
    pub key_id: Option<KeyId>,     // 定義されたキーID（オプション）
    pub x: f32,                    // x coordinate [mm]
    pub y: f32,                    // y coordinate [mm]
    pub width_u: f32,              // キー幅 [u]
    pub block_id: Option<BlockId>, // 矢印キー用のブロックID（オプション）
    pub layer: u8,                 // レイヤ番号（0=ベースレイヤ、1以上=モディファイアレイヤ）
}

/// Overall geometry
#[derive(Debug, Clone)]
pub struct Geometry {
    pub name: GeometryName,
    pub cells: Vec<Vec<Cell>>,                         // cells[row][col]
    pub homes: HashMap<Finger, (f32, f32)>,            // Finger → home coordinates [R^2]
    pub key_placements: HashMap<String, KeyPlacement>, // store all key placements
    pub max_layers: usize,                             // 最大レイヤ番号（最適化結果に応じて変動）
}

/// Candidate set for general keys (start cell and allowed widths)
#[derive(Debug, Clone)]
pub struct KeyCandidates {
    pub starts: Vec<(CellId, Vec<f32>)>, // (Start cell, width candidates)
}

/// Finger enum と文字列の変換ユーティリティ
pub fn finger_from_string(s: &str) -> Option<Finger> {
    match s {
        "LThumb" => Some(Finger::LThumb),
        "LIndex" => Some(Finger::LIndex),
        "LMiddle" => Some(Finger::LMiddle),
        "LRing" => Some(Finger::LRing),
        "LPinky" => Some(Finger::LPinky),
        "RThumb" => Some(Finger::RThumb),
        "RIndex" => Some(Finger::RIndex),
        "RMiddle" => Some(Finger::RMiddle),
        "RRing" => Some(Finger::RRing),
        "RPinky" => Some(Finger::RPinky),
        _ => None,
    }
}

pub fn finger_to_string(finger: Finger) -> &'static str {
    match finger {
        Finger::LThumb => "LThumb",
        Finger::LIndex => "LIndex",
        Finger::LMiddle => "LMiddle",
        Finger::LRing => "LRing",
        Finger::LPinky => "LPinky",
        Finger::RThumb => "RThumb",
        Finger::RIndex => "RIndex",
        Finger::RMiddle => "RMiddle",
        Finger::RRing => "RRing",
        Finger::RPinky => "RPinky",
    }
}

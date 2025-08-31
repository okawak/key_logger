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
    pub key_id: Option<KeyId>, // 定義されたキーID（オプション）
    pub x: f64,                // x coordinate [mm]
    pub y: f64,                // y coordinate [mm]
    pub width_u: f64,          // キー幅 [u]
    pub layer: u8,             // レイヤ番号（0=ベースレイヤ、1以上=モディファイアレイヤ）
}

/// Overall geometry
#[derive(Debug, Clone)]
pub struct Geometry {
    /// 配列の名前 (enum)
    pub name: GeometryName,
    /// 二次元のセル情報: cells[row][col]
    pub cells: Vec<Vec<Cell>>,
    /// 指ごとのホームポジション座標: Finger → (x_mm, y_mm)
    pub homes: HashMap<Finger, (f64, f64)>,
    /// キー配置マップ (結果が格納される): KeyId → KeyPlacement
    pub key_placements: HashMap<String, KeyPlacement>, // store all key placements
    /// 最大のレイヤ番号 (v2以降で使用)
    pub max_layers: usize,
}

/// Candidate set for general keys (start cell and allowed widths)
#[derive(Debug, Clone)]
pub struct KeyCandidates {
    pub starts: Vec<(CellId, Vec<f64>)>, // (Start cell, width candidates)
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

pub fn finger_to_string(finger: &Finger) -> &'static str {
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

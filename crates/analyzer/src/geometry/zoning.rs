use crate::{
    constants::MIDDLE_CELL,
    geometry::types::{Finger, Finger::*},
};

/// 境界値を事前計算（コンパイル時）
const BOUNDARIES: [usize; 7] = [
    MIDDLE_CELL - 16, // LPinky | LRing
    MIDDLE_CELL - 12, // LRing | LMiddle
    MIDDLE_CELL - 8,  // LMiddle | LIndex
    MIDDLE_CELL,      // LIndex | RIndex
    MIDDLE_CELL + 8,  // RIndex | RMiddle
    MIDDLE_CELL + 12, // RMiddle | RRing
    MIDDLE_CELL + 16, // RRing | RPinky
];

const FINGERS: [Finger; 8] = [
    LPinky, LRing, LMiddle, LIndex, RIndex, RMiddle, RRing, RPinky,
];

/// 親指以外の指の割り当てをx座標を用いて決める
pub fn finger_from_x(x: usize) -> Finger {
    // バイナリサーチで境界を見つける
    let index = BOUNDARIES.partition_point(|&boundary| x >= boundary);
    FINGERS[index]
}

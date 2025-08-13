use crate::constants::FINGER_X_BOUNDARY;
use crate::geometry::types::{Finger, Finger::*};

#[inline]
pub fn finger_from_x(x: usize) -> Finger {
    if x < FINGER_X_BOUNDARY[0] {
        return LPinky;
    }
    if x < FINGER_X_BOUNDARY[1] {
        return LRing;
    }
    if x < FINGER_X_BOUNDARY[2] {
        return LMiddle;
    }
    if x < FINGER_X_BOUNDARY[3] {
        return LIndex;
    }
    if x < FINGER_X_BOUNDARY[4] {
        return RIndex;
    }
    if x < FINGER_X_BOUNDARY[5] {
        return RMiddle;
    }
    if x < FINGER_X_BOUNDARY[6] {
        return RRing;
    }
    RPinky
}

use crate::{
    config::Config,
    constants::MIDDLE_CELL,
    geometry::{builders::GeometryBuilder, types::Finger},
};
use std::collections::HashMap;

/// Key definition with position and width for baseline layout
#[derive(Debug, Clone)]
pub struct CustomKeyDef {
    pub key_name: &'static str,
    pub row: usize,
    pub start_cell_offset: i32, // offset from middle cell
    pub width_u: f64,           // width in units
}

/// Baseline keyboard layout (QWERTY + typical modifiers)
/// This represents the layout used when creating the CSV frequency data
pub const BASELINE_LAYOUT: &[CustomKeyDef] = &[
    // Row 0 (bottom): Space bar and modifiers
    CustomKeyDef {
        key_name: "leftcontrol",
        row: 0,
        start_cell_offset: -27,
        width_u: 1.25,
    },
    CustomKeyDef {
        key_name: "leftmeta",
        row: 0,
        start_cell_offset: -22,
        width_u: 1.25,
    },
    CustomKeyDef {
        key_name: "leftalt",
        row: 0,
        start_cell_offset: -17,
        width_u: 1.25,
    },
    CustomKeyDef {
        key_name: "space",
        row: 0,
        start_cell_offset: -12,
        width_u: 6.25,
    },
    CustomKeyDef {
        key_name: "rightalt",
        row: 0,
        start_cell_offset: 13,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "menu", // None
        row: 0,
        start_cell_offset: 17,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "rightcontrol",
        row: 0,
        start_cell_offset: 21,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "arrowleft",
        row: 0,
        start_cell_offset: 25,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "arrowdown",
        row: 0,
        start_cell_offset: 29,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "arrowright",
        row: 0,
        start_cell_offset: 33,
        width_u: 1.0,
    },
    // Row 1: ZXCV...
    CustomKeyDef {
        key_name: "leftshift",
        row: 1,
        start_cell_offset: -27,
        width_u: 2.25,
    },
    CustomKeyDef {
        key_name: "Z",
        row: 1,
        start_cell_offset: -18,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "X",
        row: 1,
        start_cell_offset: -14,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "C",
        row: 1,
        start_cell_offset: -10,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "V",
        row: 1,
        start_cell_offset: -6,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "B",
        row: 1,
        start_cell_offset: -2,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "N",
        row: 1,
        start_cell_offset: 2,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "M",
        row: 1,
        start_cell_offset: 6,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: ",",
        row: 1,
        start_cell_offset: 10,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: ".",
        row: 1,
        start_cell_offset: 14,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "/",
        row: 1,
        start_cell_offset: 18,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "rightshift",
        row: 1,
        start_cell_offset: 22,
        width_u: 1.75,
    },
    CustomKeyDef {
        key_name: "arrowup",
        row: 1,
        start_cell_offset: 29,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "pgdown", // None
        row: 1,
        start_cell_offset: 33,
        width_u: 1.0,
    },
    // Row 2: ASDF... (home row)
    CustomKeyDef {
        key_name: "capslock",
        row: 2,
        start_cell_offset: -27,
        width_u: 1.75,
    },
    CustomKeyDef {
        key_name: "A",
        row: 2,
        start_cell_offset: -20,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "S",
        row: 2,
        start_cell_offset: -16,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "D",
        row: 2,
        start_cell_offset: -12,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "F",
        row: 2,
        start_cell_offset: -8,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "G",
        row: 2,
        start_cell_offset: -4,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "H",
        row: 2,
        start_cell_offset: 0,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "J",
        row: 2,
        start_cell_offset: 4,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "K",
        row: 2,
        start_cell_offset: 8,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "L",
        row: 2,
        start_cell_offset: 12,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: ";",
        row: 2,
        start_cell_offset: 16,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "'",
        row: 2,
        start_cell_offset: 20,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "enter",
        row: 2,
        start_cell_offset: 24,
        width_u: 2.25,
    },
    CustomKeyDef {
        key_name: "pgup", // None
        row: 2,
        start_cell_offset: 33,
        width_u: 1.0,
    },
    // Row 3: QWER...
    CustomKeyDef {
        key_name: "tab",
        row: 3,
        start_cell_offset: -27,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "Q",
        row: 3,
        start_cell_offset: -21,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "W",
        row: 3,
        start_cell_offset: -17,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "E",
        row: 3,
        start_cell_offset: -13,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "R",
        row: 3,
        start_cell_offset: -9,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "T",
        row: 3,
        start_cell_offset: -5,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "Y",
        row: 3,
        start_cell_offset: -1,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "U",
        row: 3,
        start_cell_offset: 3,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "I",
        row: 3,
        start_cell_offset: 7,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "O",
        row: 3,
        start_cell_offset: 11,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "P",
        row: 3,
        start_cell_offset: 15,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "[",
        row: 3,
        start_cell_offset: 19,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "]",
        row: 3,
        start_cell_offset: 23,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "\\",
        row: 3,
        start_cell_offset: 27,
        width_u: 1.5,
    },
    CustomKeyDef {
        key_name: "delete",
        row: 3,
        start_cell_offset: 33,
        width_u: 1.0,
    },
    // Row 4: 1234... (number row)
    CustomKeyDef {
        key_name: "`",
        row: 4,
        start_cell_offset: -27,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "1",
        row: 4,
        start_cell_offset: -23,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "2",
        row: 4,
        start_cell_offset: -19,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "3",
        row: 4,
        start_cell_offset: -15,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "4",
        row: 4,
        start_cell_offset: -11,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "5",
        row: 4,
        start_cell_offset: -7,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "6",
        row: 4,
        start_cell_offset: -3,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "7",
        row: 4,
        start_cell_offset: 1,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "8",
        row: 4,
        start_cell_offset: 5,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "9",
        row: 4,
        start_cell_offset: 9,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "0",
        row: 4,
        start_cell_offset: 13,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "-",
        row: 4,
        start_cell_offset: 17,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "=",
        row: 4,
        start_cell_offset: 21,
        width_u: 1.0,
    },
    CustomKeyDef {
        key_name: "backspace",
        row: 4,
        start_cell_offset: 25,
        width_u: 2.0,
    },
    CustomKeyDef {
        key_name: "`",
        row: 4,
        start_cell_offset: 33,
        width_u: 1.0,
    },
];

/// Determine which finger should press a key based on its position
/// Uses start_cell_offset relative to G(-4) and H(0) positions as reference
pub fn determine_finger_for_key(key_def: &CustomKeyDef) -> Finger {
    use crate::constants::U2CELL;
    use Finger::*;

    // Calculate key center offset from the middle (G/H boundary)
    let key_center_offset =
        key_def.start_cell_offset + (key_def.width_u * U2CELL as f64 / 2.0) as i32;

    // Finger assignment based on offset from G(-4)/H(0) boundary
    let base_finger = match key_center_offset {
        ..=-17 => LPinky,
        -16..=-13 => LRing,
        -12..=-9 => LMiddle,
        -8..=-1 => LIndex,
        0..=7 => RIndex,
        8..=11 => RMiddle,
        12..=15 => RRing,
        16.. => RPinky,
    };

    // Special handling for row 0 (bottom row): thumb takes over non-pinky areas
    if key_def.row == 0 {
        match base_finger {
            LPinky | RPinky => base_finger, // Pinky areas remain pinky
            LRing | LMiddle | LIndex | LThumb => LThumb, // Left side becomes left thumb
            RIndex | RMiddle | RRing | RThumb => RThumb, // Right side becomes right thumb
        }
    } else {
        base_finger
    }
}

pub struct CustomBuilder;

impl GeometryBuilder for CustomBuilder {
    fn get_fixed_key_positions(_config: &Config) -> Vec<(usize, usize, Vec<&'static str>)> {
        // Not used for baseline evaluation
        vec![]
    }

    fn build_home_positions(_config: &Config) -> HashMap<Finger, (f64, f64)> {
        use crate::constants::cell_to_key_center;
        use Finger::*;

        // Home row finger positions (same as row_stagger.rs)
        let home_finger_data = [
            (LPinky, -20, 2),  // A
            (LRing, -16, 2),   // S
            (LMiddle, -12, 2), // D
            (LIndex, -8, 2),   // F
            (LThumb, -8, 0),   // Same cell as LIndex but row 0
            (RIndex, 4, 2),    // J
            (RThumb, 4, 0),    // Same cell as RIndex but row 0
            (RMiddle, 8, 2),   // K
            (RRing, 12, 2),    // L
            (RPinky, 16, 2),   // ;
        ];

        home_finger_data
            .iter()
            .map(|&(finger, offset, row)| {
                let cell = (MIDDLE_CELL as i32 + offset) as usize;
                (finger, cell_to_key_center(row, cell, 1.0))
            })
            .collect()
    }
}

/// Key layout settings
pub const MAX_ROW: usize = 6; // (max) [u] (only suit for row stagger/otho layout)
pub const MAX_COL_CELLS: usize = 80; // (max) 20 [u] x 4 = 80 [cell]

/// Use key setting
pub const MAX_DIGIT: u8 = 9;
pub const MAX_NUMPAD_DIGIT: u8 = 9;
pub const DEFAULT_FKEYS_MAX: u8 = 12;

/// unit conversion u -> mm
pub const U2MM: f64 = 19.0; // u -> mm (f64 for Fitts calculation)
pub const U2CELL: usize = 4; // u -> cell
pub const U2PX: f32 = 60.0; // u -> px (for visualization)

/// Expected headers in CSV files
pub const EXPECTED_KEY_HEADER: &str = "Key"; // Key column header
pub const EXPECTED_COUNT_HEADER: &str = "Count"; // Count column header

/// Finger region (cell unit)
pub const FINGER_X_BOUNDARY: [usize; 7] = [24, 28, 32, 40, 48, 52, 56];

/// Visualization
pub const MARGIN: f32 = 24.0; // margin [px]
pub const LEGEND_WIDTH: f32 = 320.0; // legend width [px]
pub const FONT_SIZE: f32 = 16.0; // font size [px]

/// calculate cell start position \[cell\] to \[mm\]
/// - row: u unit
/// - col: cell unit
#[inline]
pub fn cell_to_cordinate(row: usize, col: usize) -> (f32, f32) {
    let x = (col as f32 / U2CELL as f32) * U2MM as f32;
    let y = row as f32 * U2MM as f32;
    (x, y)
}

/// calculate center key position \[cell\] to \[mm\] (assume vertical 1u size)
/// - row: u unit
/// - col: cell unit
/// - width: \[u\]
#[inline]
pub fn cell_to_key_center(row: usize, col: usize, width: f32) -> (f32, f32) {
    let (mut x, mut y) = cell_to_cordinate(row, col);
    x += width / 2.0 * U2MM as f32;
    y += 0.5 * U2MM as f32; // キーの中心位置（0.5u offset for center）
    (x, y)
}

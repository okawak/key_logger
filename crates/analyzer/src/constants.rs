/// Key layout settings
pub const MAX_ROW_CELLS: usize = 24; // (max) 6 rows [u] x 4 = 24 [cell]
pub const MAX_COL_CELLS: usize = 80; // (max) 20 [u] x 4 = 80 [cell]

/// Use key setting
pub const MAX_DIGIT: u8 = 9;
pub const MAX_NUMPAD_DIGIT: u8 = 9;
pub const DEFAULT_FKEYS_MAX: u8 = 12;

/// unit conversion u -> mm
pub const U2MM: f64 = 19.0; // u -> mm
pub const CELL2U: f32 = 4.0; // cell -> u

/// Expected headers in CSV files
pub const EXPECTED_KEY_HEADER: &str = "Key"; // Key column header
pub const EXPECTED_COUNT_HEADER: &str = "Count"; // Count column header

/// Finger region (cell unit)
pub const FINGER_X_BOUNDARY: [usize; 7] = [12, 16, 20, 28, 36, 40, 44];

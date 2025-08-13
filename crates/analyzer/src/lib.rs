pub mod constants;
pub mod csv_reader;
pub mod error;
pub mod geometry;
pub mod keys;
pub mod optimize;

pub use constants::{
    DEFAULT_FKEYS_MAX, EXPECTED_COUNT_HEADER, EXPECTED_KEY_HEADER, MAX_DIGIT, MAX_NUMPAD_DIGIT,
};
pub use csv_reader::{
    KeyFreq, create_fallback_data, read_key_freq_csv, read_key_freq_from_directory,
};
pub use error::KbOptError;
pub use geometry::Geometry;
pub use geometry::visualization::{
    DebugRenderOptions, render_optimized_layout, save_optimized_layout,
    save_optimized_layout_to_figs,
};
pub use keys::{ArrowKey, KeyId, SymbolKey, all_movable_keys, allowed_widths};

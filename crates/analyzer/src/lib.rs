pub mod csv_reader;
pub mod error;
pub mod geometry;
pub mod keys;
pub mod optimize;

pub use csv_reader::{KeyFreq, read_key_freq_csv, read_key_freq_from_directory};
pub use error::KbOptError;
pub use geometry::Geometry;
pub use geometry::vis::{render_optimized_layout, save_optimized_layout_to_figs, save_optimized_layout, DebugRenderOptions};
pub use keys::{ArrowKey, KeyId, SymbolKey, all_movable_keys, allowed_widths};

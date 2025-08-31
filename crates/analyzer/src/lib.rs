pub mod config;
pub mod constants;
pub mod csv_reader;
pub mod error;
pub mod geometry;
pub mod keys;
pub mod optimize;

pub use config::Config;
pub use constants::{
    DEFAULT_FKEYS_MAX, EXPECTED_COUNT_HEADER, EXPECTED_KEY_HEADER, MAX_DIGIT, MAX_NUMPAD_DIGIT,
    MAX_ROW, MIN_ROW,
};
pub use csv_reader::{KeyFreq, read_key_freq};
pub use error::KbOptError;
pub use geometry::{Geometry, GeometryName, save_layout};
pub use keys::{ArrowKey, KeyId, SymbolKey};
pub use optimize::{Solution, solve_layout};

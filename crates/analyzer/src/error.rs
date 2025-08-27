use thiserror::Error;

pub type Result<T> = std::result::Result<T, KbOptError>;

#[derive(Debug, Error)]
pub enum KbOptError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid CSV Header: {0}")]
    CsvHeader(String),

    #[error("Invalid CSV row {row}: expected at least 2 columns, got {got}")]
    CsvRow { row: usize, got: usize },

    #[error("Unknown key label at row {row}: {label}")]
    UnknownKey { row: usize, label: String },

    #[error("Invalid count at row {row}: {value}")]
    CountParse {
        row: usize,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error("Optimization solver error: {0}")]
    Solver(String),

    #[error("Model construction error: {message}")]
    Model { message: String },

    #[error("Invalid geometry: {message}")]
    Geometry { message: String },

    #[error("Key placement error: {message}")]
    Placement { message: String },

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Other error: {0}")]
    Other(String),
}

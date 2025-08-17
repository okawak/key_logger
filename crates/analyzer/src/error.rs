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
    SolverError(String),

    #[error("Model construction error: {message}")]
    ModelError { message: String },

    #[error("Invalid geometry: {message}")]
    GeometryError { message: String },

    #[error("Key placement error: {message}")]
    PlacementError { message: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<toml::de::Error> for KbOptError {
    fn from(err: toml::de::Error) -> Self {
        KbOptError::ConfigError(format!("TOML parse error: {}", err))
    }
}

impl From<serde_json::Error> for KbOptError {
    fn from(err: serde_json::Error) -> Self {
        KbOptError::IoError(format!("JSON error: {}", err))
    }
}

use std::{error::Error as StdError, io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum KeyLoggerError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Failed to acquire statistics lock")]
    StatisticsLockError,

    #[error("Platform not supported")]
    PlatformNotSupported,

    #[error("Signal handling error")]
    SignalHandling {
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },

    #[error("Failed to create directory {path}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to create file {path}")]
    CreateFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to write file {path}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),

    #[cfg(windows)]
    #[error(transparent)]
    Ctrlc(#[from] ctrlc::Error),
}

pub type Result<T> = std::result::Result<T, KeyLoggerError>;

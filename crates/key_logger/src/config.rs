use crate::error::{KeyLoggerError, Result};
use std::{env, path::PathBuf};

const ENV_KEY_OUTPUT_DIR: &str = "KEY_LOGGER_OUTPUT_DIR";

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub output_dir: Option<PathBuf>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Set default output directory to "./csv"
        let default_output_dir = PathBuf::from("csv");

        if let Ok(output_dir) = env::var(ENV_KEY_OUTPUT_DIR)
            && !output_dir.trim().is_empty()
        {
            let path = PathBuf::from(output_dir);

            // If the path already exists but is not a directory, reject early.
            if path.exists() && !path.is_dir() {
                return Err(KeyLoggerError::InvalidConfiguration(format!(
                    "Output path is not a directory: {}",
                    path.display()
                )));
            }
            config.output_dir = Some(path);
        } else {
            // Use default csv directory when environment variable is not set
            config.output_dir = Some(default_output_dir);
        }
        Ok(config)
    }
}

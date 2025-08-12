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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.output_dir.is_none());
    }

    #[test]
    fn test_from_env_default() {
        // Store original values
        let orig_output_dir = env::var("KEY_LOGGER_OUTPUT_DIR").ok();

        // Clear environment variables
        unsafe {
            env::remove_var("KEY_LOGGER_OUTPUT_DIR");
        }

        let config = Config::from_env().unwrap();
        // Should now default to "csv" directory
        assert_eq!(config.output_dir, Some(PathBuf::from("csv")));

        // Restore original values
        if let Some(value) = orig_output_dir {
            unsafe {
                env::set_var("KEY_LOGGER_OUTPUT_DIR", value);
            }
        }
    }

    #[test]
    fn test_from_env_with_valid_output_dir() {
        // Store original value for cleanup
        let orig_output_dir = env::var("KEY_LOGGER_OUTPUT_DIR").ok();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        unsafe {
            env::set_var("KEY_LOGGER_OUTPUT_DIR", &temp_path);
        }

        let config = Config::from_env().unwrap();
        assert_eq!(config.output_dir, Some(temp_path));

        // Cleanup
        unsafe {
            env::remove_var("KEY_LOGGER_OUTPUT_DIR");
            if let Some(value) = orig_output_dir {
                env::set_var("KEY_LOGGER_OUTPUT_DIR", value);
            }
        }
    }

    #[test]
    fn test_from_env_with_empty_string() {
        let orig_output_dir = env::var("KEY_LOGGER_OUTPUT_DIR").ok();

        unsafe {
            env::set_var("KEY_LOGGER_OUTPUT_DIR", "");
        }

        let config = Config::from_env().unwrap();
        // Should default to "csv" directory when empty string
        assert_eq!(config.output_dir, Some(PathBuf::from("csv")));

        // Cleanup
        unsafe {
            env::remove_var("KEY_LOGGER_OUTPUT_DIR");
            if let Some(value) = orig_output_dir {
                env::set_var("KEY_LOGGER_OUTPUT_DIR", value);
            }
        }
    }

    #[test]
    fn test_from_env_with_whitespace_only() {
        let orig_output_dir = env::var("KEY_LOGGER_OUTPUT_DIR").ok();

        unsafe {
            env::set_var("KEY_LOGGER_OUTPUT_DIR", "   \t\n   ");
        }

        let config = Config::from_env().unwrap();
        // Should default to "csv" directory when whitespace only
        assert_eq!(config.output_dir, Some(PathBuf::from("csv")));

        // Cleanup
        unsafe {
            env::remove_var("KEY_LOGGER_OUTPUT_DIR");
            if let Some(value) = orig_output_dir {
                env::set_var("KEY_LOGGER_OUTPUT_DIR", value);
            }
        }
    }

    #[test]
    fn test_cross_platform_path_handling() {
        let orig_output_dir = env::var("KEY_LOGGER_OUTPUT_DIR").ok();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Test with different path formats
        let path_str = temp_path.to_string_lossy();
        unsafe {
            env::set_var("KEY_LOGGER_OUTPUT_DIR", path_str.as_ref());
        }

        let config = Config::from_env().unwrap();
        assert!(config.output_dir.is_some());

        let configured_path = config.output_dir.unwrap();

        // Verify paths are equivalent (handle different separators)
        assert_eq!(
            configured_path.canonicalize().unwrap(),
            temp_path.canonicalize().unwrap()
        );

        // Cleanup
        unsafe {
            env::remove_var("KEY_LOGGER_OUTPUT_DIR");
            if let Some(value) = orig_output_dir {
                env::set_var("KEY_LOGGER_OUTPUT_DIR", value);
            }
        }
    }

    #[test]
    fn test_default_csv_directory() {
        // Store original value for cleanup
        let orig_output_dir = env::var("KEY_LOGGER_OUTPUT_DIR").ok();

        // Clear environment variable to test default behavior
        unsafe {
            env::remove_var("KEY_LOGGER_OUTPUT_DIR");
        }

        let config = Config::from_env().unwrap();

        // Should default to "csv" directory
        assert!(config.output_dir.is_some());
        assert_eq!(config.output_dir.unwrap(), PathBuf::from("csv"));

        // Cleanup
        if let Some(value) = orig_output_dir {
            unsafe {
                env::set_var("KEY_LOGGER_OUTPUT_DIR", value);
            }
        }
    }
}

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

        if let Ok(output_dir) = env::var(ENV_KEY_OUTPUT_DIR) {
            if !output_dir.trim().is_empty() {
                let path = PathBuf::from(output_dir);

                // If the path already exists but is not a directory, reject early.
                if path.exists() && !path.is_dir() {
                    return Err(KeyLoggerError::InvalidConfiguration(format!(
                        "Output path is not a directory: {}",
                        path.display()
                    )));
                }

                config.output_dir = Some(path);
            }
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
        assert!(config.output_dir.is_none());

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
        assert!(config.output_dir.is_none());

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
        assert!(config.output_dir.is_none());

        // Cleanup
        unsafe {
            env::remove_var("KEY_LOGGER_OUTPUT_DIR");
            if let Some(value) = orig_output_dir {
                env::set_var("KEY_LOGGER_OUTPUT_DIR", value);
            }
        }
    }

    #[test]
    fn test_from_env_with_file_path_error() {
        let orig_output_dir = env::var("KEY_LOGGER_OUTPUT_DIR").ok();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a file (not a directory)
        std::fs::write(&file_path, "test").unwrap();

        unsafe {
            env::set_var("KEY_LOGGER_OUTPUT_DIR", &file_path);
        }

        let result = Config::from_env();
        assert!(result.is_err());

        if let Err(crate::error::KeyLoggerError::InvalidConfiguration(msg)) = result {
            assert!(msg.contains("not a directory"));
        } else {
            panic!("Expected InvalidConfiguration error");
        }

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
}

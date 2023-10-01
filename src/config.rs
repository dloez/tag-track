//! This module provides functions for working with configuration files.
//!
//! The `Config` struct represents the structure of the configuration file.
//! The `is_config_available` function checks if a configuration file exists in the current directory.
//! The `parse_config_file` function parses a configuration file and returns a `Config` object.
//!

use crate::error::Error;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

/// The `Config` struct represents the structure of the configuration file.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The tag pattern used to extract the version number from Git tags.
    pub tag_pattern: Option<String>,
}

impl Config {
    /// Creates a new `Config` object with default values.
    pub fn new() -> Self {
        Self { tag_pattern: None }
    }
}

/// Reads the contents of a file into a string.
fn read_file(path: &PathBuf) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Checks if a configuration file exists in the current directory.
///
/// Returns the path to the configuration file if it exists, or `None` otherwise.
pub fn is_config_available() -> Option<PathBuf> {
    const CONFIG_FILE_NAMES: [&str; 2] = ["track.yml", "track.yaml"];

    for file_name in &CONFIG_FILE_NAMES {
        let path = PathBuf::from(file_name);
        if let Ok(metadata) = fs::metadata(&path) {
            if metadata.is_file() {
                return Some(path);
            }
        }
    }

    None
}

/// Parses a configuration file and returns a `Config` object.
///
/// # Arguments
///
/// * `file_path` - The path to the configuration file.
///
/// # Errors
///
/// Returns an `Error` object if the file cannot be read or parsed.
///
pub fn parse_config_file(file_path: PathBuf) -> Result<Config, Error> {
    let contents = read_file(&file_path)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

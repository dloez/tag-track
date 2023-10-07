//! This module provides functions for working with configuration files.
//!
//! The `Config` struct represents the structure of the configuration file.
//! The `is_config_available` function checks if a configuration file exists in the current directory.
//! The `parse_config_file` function parses a configuration file and returns a `Config` object.
//!

use crate::error::Error;
use crate::version::IncrementKind;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
    vec,
};

/// Default Regex pattern used to validate tags and extract the version from it.
const DEFAULT_TAG_PATTERN: &str = r"(.*)";

/// Default Regex pattern used to validate conventional commits and extract the required fields from it.
pub const DEFAULT_COMMIT_PATTERN: &str =
    r"^(?<type>[a-zA-Z]*)(?<scope>\(.*\))?(?<breaking>!)?:(?<description>[\s\S]*)$";

fn get_default_bump_rules() -> Vec<BumpRule> {
    vec![
        BumpRule {
            bump: IncrementKind::Patch,
            types: Some(vec![String::from("fix"), String::from("style")]),
            scopes: None,
            if_breaking_field: None,
            if_breaking_description: None,
        },
        BumpRule {
            bump: IncrementKind::Minor,
            types: Some(vec![
                String::from("feat"),
                String::from("refactor"),
                String::from("perf"),
            ]),
            scopes: None,
            if_breaking_field: None,
            if_breaking_description: None,
        },
        BumpRule {
            bump: IncrementKind::Major,
            types: None,
            scopes: None,
            if_breaking_field: Some(true),
            if_breaking_description: Some(true),
        },
    ]
}

/// Type used to parse the configuration file.
#[derive(Debug, Deserialize)]
pub struct ParsedConfig {
    /// The tag pattern used to extract the version number from Git tags.
    pub tag_pattern: Option<String>,

    /// The commit pattern used validate and collect information from conventional commits.
    /// The pattern should expose the following named capture groups:
    /// - `type`: The type of the commit. Required.
    /// - `scope`: The scope of the commit.
    /// - `breaking`: The breaking change indicator of the commit, normally it is the `!` char in the type.
    /// - `description`: The description of the commit. Required.
    pub commit_pattern: Option<String>,

    /// Rules for bumping the version number.
    pub bump_rules: Option<Vec<BumpRule>>,
}

/// Type to represent the rules for bumping the version number.
#[derive(Debug, Deserialize)]
pub struct BumpRule {
    /// Which version field should be bumped if the rule triggers.
    pub bump: IncrementKind,

    /// Which commit types trigger the rule.
    pub types: Option<Vec<String>>,

    /// Which scopes trigger the rule.
    pub scopes: Option<Vec<String>>,

    /// Use `true` if you want the rule to trigger if the field `breaking` in the commit pattern matches.
    pub if_breaking_field: Option<bool>,

    /// Use `true` if you want the rule to trigger if the commit description includes the strings 'BREAKING CHANGE' or 'BREAKING-CHANGE'.
    pub if_breaking_description: Option<bool>,
}

/// Type used to add default fields to the missing configuration field fields.
#[derive(Debug)]
pub struct Config {
    /// The tag pattern used to extract the version number from Git tags.
    pub tag_pattern: String,

    /// The commit pattern used validate and collect information from conventional commits.
    /// The pattern should expose the following named capture groups:
    /// - `type`: The type of the commit. Required.
    /// - `scope`: The scope of the commit.
    /// - `breaking`: The breaking change indicator of the commit, normally it is the `!` char in the type.
    /// - `description`: The description of the commit. Required.
    pub commit_pattern: String,

    /// Rules for bumping the version number.
    pub bump_rules: Vec<BumpRule>,
}

impl From<ParsedConfig> for Config {
    /// Convert from `ParsedConfig` to `Config`. If any of the fields are missing, the default values are used.
    fn from(parsed_config: ParsedConfig) -> Self {
        let tag_pattern = match parsed_config.tag_pattern {
            Some(tag_pattern) => tag_pattern,
            None => DEFAULT_TAG_PATTERN.to_owned(),
        };

        let commit_pattern = match parsed_config.commit_pattern {
            Some(commit_pattern) => commit_pattern,
            None => DEFAULT_COMMIT_PATTERN.to_owned(),
        };

        let bump_rules: Vec<BumpRule> = match parsed_config.bump_rules {
            Some(bump_rules) => bump_rules,
            None => get_default_bump_rules(),
        };

        Self {
            tag_pattern,
            commit_pattern,
            bump_rules,
        }
    }
}

impl Config {
    /// Create a new instance of `Config` wiht detaulf values.
    pub fn new() -> Config {
        Self {
            tag_pattern: DEFAULT_TAG_PATTERN.to_owned(),
            commit_pattern: DEFAULT_COMMIT_PATTERN.to_owned(),
            bump_rules: get_default_bump_rules(),
        }
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
    let parsed_config: ParsedConfig = serde_yaml::from_str(&contents)?;
    Ok(Config::from(parsed_config))
}

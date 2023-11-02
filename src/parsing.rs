//! This module includes utilities to parse conventional commits and tags.
//!

use crate::error::{Error, ErrorKind};
use regex::Regex;
use semver::Version;

/// Regex capturing group name for the type of the commit.
const TYPE_CAPTURING_GROUP_NAME: &str = "type";
/// Regex capturing group name for the scope of the commit.
const SCOPE_CAPTURING_GROUP_NAME: &str = "scope";
/// Regex capturing group name for the breaking change indicator of the commit.
const BREAKING_CAPTURING_GROUP_NAME: &str = "breaking";
/// Regex capturing group name for the description of the commit.
const DESCRIPTION_CAPTURING_GROUP_NAME: &str = "description";
/// Regex capturing group name for the version inside a tag.
const VERSION_CAPTURING_GROUP_NAME: &str = "version";

/// Type to represent the sections of a conventional commit message.
#[derive(Debug)]
pub struct CommitDetails {
    /// The type of the commit.
    pub commit_type: String,

    /// The scope of the commit.
    pub scope: Option<String>,

    /// If the commit includes a breaking change. Typically this is true if the commit type includes the `!` char.
    pub breaking: bool,

    /// The description of the conventional commit.
    pub description: String,
}

/// Extracts the commit details from a commit message.
///
/// # Arguments
///
/// * `commit_message` - Commit message that will be parsed.
///
/// * `commit_pattern` - Pattern that will be used to parse the conventional commit.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::InvalidRegexPattern` if the given `commit_pattern`
/// is not a valid regex pattern.
///
pub fn parse_commit_details(
    commit_message: &str,
    commit_pattern: &str,
) -> Result<Option<CommitDetails>, Error> {
    let re = match Regex::new(commit_pattern) {
        Ok(re) => re,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidRegexPattern,
                Some(format!("{} - {}", commit_pattern, error.to_string().as_str()).as_str()),
            ))
        }
    };

    let captures = match re.captures(commit_message) {
        Some(captures) => captures,
        None => return Ok(None),
    };

    let commit_type = match captures.name(TYPE_CAPTURING_GROUP_NAME) {
        Some(found_match) => found_match.as_str().trim().to_string(),
        None => return Ok(None),
    };

    let scope = captures
        .name(SCOPE_CAPTURING_GROUP_NAME)
        .map(|found_match| {
            found_match
                .as_str()
                .replace(['(', ')'], "")
                .trim()
                .to_string()
        });

    let breaking = captures.name(BREAKING_CAPTURING_GROUP_NAME).is_some();

    let description = match captures.name(DESCRIPTION_CAPTURING_GROUP_NAME) {
        None => return Ok(None),
        Some(found_match) => {
            if found_match.is_empty() {
                return Ok(None);
            }
            found_match.as_str().trim().to_string()
        }
    };

    Ok(Some(CommitDetails {
        commit_type,
        scope,
        breaking,
        description,
    }))
}

/// Type to represent the sections of a tag.
#[derive(Debug, Clone)]
pub struct TagDetails {
    /// The version inside the tag.
    pub version: Version,

    /// The scope of the version.
    pub scope: Option<String>,
}

/// Extracts the tag details from a tag name.
///
/// # Arguments
///
/// * `tag_name` - Tag name that will be parsed.
///
/// * `tag_pattern` - Pattern that will be used to parse the tag.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::InvalidRegexPattern` if the given `tag_pattern`
/// is not a valid regex pattern.
///
pub fn parse_tag_details(tag_name: &str, tag_pattern: &str) -> Result<Option<TagDetails>, Error> {
    let re = match Regex::new(tag_pattern) {
        Ok(re) => re,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidRegexPattern,
                Some(format!("{} - {}", tag_pattern, error.to_string().as_str()).as_str()),
            ))
        }
    };

    let captures = match re.captures(tag_name) {
        Some(captures) => captures,
        None => return Ok(None),
    };

    let version = match captures.name(VERSION_CAPTURING_GROUP_NAME) {
        Some(found_match) => {
            let version = found_match.as_str().trim().to_string();

            Version::parse(&version)?
        }
        None => return Ok(None),
    };

    let scope = captures
        .name(SCOPE_CAPTURING_GROUP_NAME)
        .map(|found_match| {
            found_match
                .as_str()
                .replace(['(', ')'], "")
                .trim()
                .to_string()
        });

    Ok(Some(TagDetails { version, scope }))
}

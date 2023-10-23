use crate::error::{Error, ErrorKind};
use crate::git::{Commit, Tag};
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
/// * `commit` - Commit message that will be parsed.
///
/// * `commit_pattern` - Pattern that will be used to parse the conventional commit.
///
/// # Errors
///
/// Returns `error::Error` with the type of `error::ErrorKind::InvalidCommitPattern` if the given Regex pattern for the commit is not a valid.
///
pub fn extract_commit_details(
    commit: &Commit,
    commit_pattern: &str,
) -> Result<CommitDetails, Error> {
    let re = match Regex::new(commit_pattern) {
        Ok(re) => re,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidRegexPattern,
                Some(format!("{} - {}", commit_pattern, error.to_string().as_str()).as_str()),
            ))
        }
    };

    let captures = match re.captures(&commit.message) {
        Some(captures) => captures,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some(
                    format!(
                        "commit {} does not match pattern {}",
                        commit.message, commit_pattern
                    )
                    .as_str(),
                ),
            ))
        }
    };

    let commit_type = match captures.name(TYPE_CAPTURING_GROUP_NAME) {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some("missing commit type"),
            ))
        }
        Some(found_match) => found_match.as_str().trim().to_string(),
    };

    let scope = match captures.name(SCOPE_CAPTURING_GROUP_NAME) {
        None => None,
        Some(found_match) => Some(
            found_match
                .as_str()
                .replace(['(', ')'], "")
                .trim()
                .to_string(),
        ),
    };

    let breaking = captures.name(BREAKING_CAPTURING_GROUP_NAME).is_some();

    let description = match captures.name(DESCRIPTION_CAPTURING_GROUP_NAME) {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some("missing commit description"),
            ))
        }
        Some(found_match) => {
            if found_match.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidCommitPattern,
                    Some("missing commit description"),
                ));
            }
            found_match.as_str().trim().to_string()
        }
    };

    Ok(CommitDetails {
        commit_type,
        scope,
        breaking,
        description,
    })
}

/// Type to represent the sections of a tag.
#[derive(Debug, Clone)]
pub struct TagDetails {
    /// The version inside the tag.
    pub version: Version,

    /// The scope of the version.
    pub scope: Option<String>,
}

pub fn extract_tag_details(tag: &Tag, tag_pattern: &str) -> Result<TagDetails, Error> {
    let re = match Regex::new(tag_pattern) {
        Ok(re) => re,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidRegexPattern,
                Some(format!("{} - {}", tag_pattern, error.to_string().as_str()).as_str()),
            ))
        }
    };

    let captures = match re.captures(&tag.name) {
        Some(captures) => captures,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidTagPattern,
                Some(format!("tag {} does not match pattern {}", tag.name, tag_pattern).as_str()),
            ))
        }
    };

    let version = match captures.name(VERSION_CAPTURING_GROUP_NAME) {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidTagPattern,
                Some("missing version inside tag"),
            ))
        }
        Some(found_match) => {
            let version = found_match.as_str().trim().to_string();
            let version = Version::parse(&version)?;
            version
        }
    };

    let scope = match captures.name(SCOPE_CAPTURING_GROUP_NAME) {
        None => None,
        Some(found_match) => Some(
            found_match
                .as_str()
                .replace(['(', ')'], "")
                .trim()
                .to_string(),
        ),
    };

    Ok(TagDetails { version, scope })
}

use crate::error::{Error, ErrorKind};
use crate::git::Commit;
use regex::Regex;

/// Type to represent the sections of a conventional commit message.
#[derive(Debug)]
pub struct CommitDetails {
    /// The type of the commit.
    pub commit_type: String,

    /// The scope of the commit.
    pub scope: String,

    /// If the commit includes a breaking change. Typically this is true if the commit type includes the `!` char.
    pub breaking: bool,

    /// The description of the conventional commit.
    pub description: String,
}

/// Regex field name for the type of the commit.
const TYPE_FIELD: &str = "type";
/// Regex field name for the scope of the commit.
const SCOPE_FIELD: &str = "scope";
/// Regex field name for the breaking change indicator of the commit.
const BREAKING_FIELD: &str = "breaking";
/// Regex field name for the description of the commit.
const DESCRIPTION_FIELD: &str = "description";

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

    let commit_type = match captures.name(TYPE_FIELD) {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some("missing commit type"),
            ))
        }
        Some(found_match) => found_match.as_str().trim().to_string(),
    };

    let scope = match captures.name(SCOPE_FIELD) {
        None => "".to_string(),
        Some(found_match) => found_match
            .as_str()
            .replace(['(', ')'], "")
            .trim()
            .to_string(),
    };

    let breaking = captures.name(BREAKING_FIELD).is_some();

    let description = match captures.name(DESCRIPTION_FIELD) {
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

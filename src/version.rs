//! Module containing version related help functions and types to define version increments.
//!
//! Version increments follow the Semantic Versioning 2.0
//!

use crate::{
    config::BumpRule,
    error::{Error, ErrorKind},
    git::Commit,
};
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};
use serde::Deserialize;

/// Types for different version.
/// The increment types follow the Semantic Version specification.
#[derive(Eq, PartialEq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncrementKind {
    /// Increment the major section of a version.
    Major,
    /// Increment the minor section of a version.
    Minor,
    /// Increment the patch section of a version.
    Patch,
}

/// Increments the patch section of a `semver::Version`. This also empties the pre-release
/// and build sections.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
pub fn increment_patch(version: &mut Version) {
    version.patch += 1;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
}

/// Increments the minor section of a `semver::Version`. This also empties the pre-release
/// and build sections, and sets the patch section to 0.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
pub fn increment_minor(version: &mut Version) {
    version.minor += 1;
    version.patch = 0;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
}

/// Increments the major section of a `semver::Version`. This also empties the pre-release
/// and build sections, and sets the patch and minor section to 0.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
pub fn increment_major(version: &mut Version) {
    version.major += 1;
    version.minor = 0;
    version.patch = 0;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
}

/// Increments the given version based on the given rules. The function returns the increment kind that was used.
/// If no increment was done, the function returns `None`.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
/// * `rules` - Rules that will be used to determine the increment kind.
///
/// * `commits` - Commits that will be used to determine the increment kind.
///
pub fn bump_version(
    version: &mut Version,
    rules: &Vec<BumpRule>,
    commits: &Vec<Commit>,
    commit_pattern: &str,
) -> Option<IncrementKind> {
    let mut increment_kind: Option<IncrementKind> = None;
    'commits: for commit in commits {
        let commit_sections = extract_commit_details(&commit, commit_pattern).unwrap();
        for rule in rules {
            let mut bump = false;

            // Check commit type
            if let Some(types) = &rule.types {
                if types.contains(&commit_sections.commit_type) {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check commit scope
            if let Some(scopes) = &rule.scopes {
                if scopes.contains(&commit_sections.scope) {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check additional chars in type
            if let Some(additional_chars) = &rule.str_in_type {
                if commit_sections
                    .commit_type_additional_chars
                    .contains(additional_chars)
                {
                    bump = true;
                } else {
                    continue;
                }
            }

            if bump {
                increment_kind = match &rule.bump {
                    IncrementKind::Major => Some(IncrementKind::Major),
                    IncrementKind::Minor => Some(IncrementKind::Minor),
                    IncrementKind::Patch => {
                        if increment_kind.is_some() {
                            continue;
                        }
                        Some(IncrementKind::Minor)
                    }
                };

                if let Some(IncrementKind::Major) = increment_kind {
                    break 'commits;
                }
            }
        }
    }

    match increment_kind {
        Some(IncrementKind::Major) => increment_major(version),
        Some(IncrementKind::Minor) => increment_minor(version),
        Some(IncrementKind::Patch) => increment_patch(version),
        None => {}
    };

    increment_kind
}

/// Type to represent the sections of a conventional commit message.
#[derive(Debug)]
struct CommitDetails {
    /// The type of the commit.
    commit_type: String,

    /// The scope of the commit.
    scope: String,

    /// If the commit includes a breaking change. Typically this is true if the commit type includes the `!` char.
    breaking: bool,

    /// The description of the conventional commit.
    _description: String,
}

const TYPE_FIELD: &str = "type";
const SCOPE_FIELD: &str = "scope";
const BREAKING_FIELD: &str = "breaking";
const DESCRIPTION_FIELD: &str = "description";

/// Extracts the commit details from a commit message.
///
/// # Arguments
///
/// * `commit` - Commit message that will be parsed.
///
/// * `commit_pattern` - Pattern that will be used to parse the conventional commit.
///
fn extract_commit_details(commit: &Commit, commit_pattern: &str) -> Result<CommitDetails, Error> {
    let re = match Regex::new(commit_pattern) {
        Ok(re) => re,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some(error.to_string().as_str()),
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
        Some(found_match) => found_match.as_str().trim().to_string(),
    };

    let breaking = match captures.name(BREAKING_FIELD) {
        None => false,
        Some(_) => true,
    };

    let description = match captures.name(DESCRIPTION_FIELD) {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some("missing commit description"),
            ))
        }
        Some(found_match) => found_match.as_str().trim().to_string(),
    };

    Ok(CommitDetails {
        commit_type,
        scope,
        breaking,
        _description: description,
    })
}

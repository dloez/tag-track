//! Module containing version related help functions and types to define version increments.
//!
//! Version increments follow the Semantic Versioning 2.0
//!

use crate::{
    config::BumpRule,
    error::{Error, ErrorKind},
    git::Commit,
    parsing::extract_commit_details,
};
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};

/// Types for different version.
/// The increment types follow the Semantic Version specification.
#[derive(Eq, PartialEq, Hash, Debug, Deserialize, Serialize)]
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
/// * `commit_pattern` - Pattern that will be used to parse the conventional commit.
///
/// # Errors
///
/// Returns `error::Error` with the type of `error::ErrorKind::InvalidCommitPattern` if the given Regex pattern for the commit is not a valid.
///
pub fn bump_version<'a>(
    version: &mut Version,
    rules: &Vec<BumpRule>,
    commits: &'a Vec<Commit>,
    commit_pattern: &str,
) -> Result<(Option<IncrementKind>, Vec<&'a String>), Error> {
    let mut skipped_commits: Vec<&String> = Vec::new();
    let mut increment_kind: Option<IncrementKind> = None;
    'commits: for commit in commits {
        let commit_details = match extract_commit_details(commit, commit_pattern) {
            Ok(commit_details) => commit_details,
            Err(error) => match error.kind {
                ErrorKind::InvalidCommitPattern => {
                    skipped_commits.push(&commit.sha);
                    continue;
                }
                _ => return Err(error),
            },
        };

        for rule in rules {
            let mut bump = false;

            // Check commit type
            if let Some(types) = &rule.types {
                if types.contains(&commit_details.commit_type) {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check commit scope
            if let Some(scopes) = &rule.scopes {
                if scopes.contains(&commit_details.scope) {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check breaking field
            if let Some(if_breaking_field) = &rule.if_breaking_field {
                if *if_breaking_field && commit_details.breaking {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check breaking description
            if let Some(if_breaking_description) = &rule.if_breaking_description {
                if *if_breaking_description
                    && (commit_details.description.contains("BREAKING CHANGE")
                        || commit_details.description.contains("BREAKING-CHANGE"))
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

    Ok((increment_kind, skipped_commits))
}

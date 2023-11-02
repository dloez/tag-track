//! Module containing version related help functions and types to define version increments.
//!
//! Version increments follow the Semantic Versioning 2.0
//!

use crate::{config::BumpRule, git::Commit};
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

/// Calculate which kind of increment is produced by a commit based on the given rules.
///
/// # Arguments
///
/// * `commit` - Commit to calculate the increment.
///
/// * `rules` - Rules to calculate the increment.
///
pub fn calculate_increment(commit: &Commit, rules: &[BumpRule]) -> Option<IncrementKind> {
    let commit_details = match &commit.details {
        Some(details) => details,
        None => return None,
    };

    let mut increment_kind: Option<IncrementKind> = None;
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
            if commit_details.scope.is_some()
                && scopes.contains(commit_details.scope.as_ref().unwrap())
            {
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
                    Some(IncrementKind::Patch)
                }
            };

            if let Some(IncrementKind::Major) = increment_kind {
                return increment_kind;
            }
        }
    }

    increment_kind
}

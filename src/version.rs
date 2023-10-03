//! Module containing version related help functions and types to define version increments.
//!
//! Version increments follow the Semantic Versioning 2.0
//!

use crate::config::BumpRule;
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
    commits: &Vec<String>,
) -> Option<IncrementKind> {
    let mut increment_kind: Option<IncrementKind> = None;
    'commits: for commit in commits {
        let commit_sections = extract_commit_sections(commit);
        for rule in rules {
            let mut bump = false;

            // Check commit type
            if let Some(types) = &rule.types {
                bump = false;
                if types.contains(&commit_sections.commit_type) {
                    bump = true;
                }
            }

            // Check commit scope
            if let Some(scopes) = &rule.scopes {
                bump = false;
                if scopes.contains(&commit_sections.scope) {
                    bump = true;
                }
            }

            // Check additional chars in type
            if let Some(additional_chars) = &rule.str_in_type {
                bump = false;
                if commit_sections
                    .commit_type_additional_chars
                    .contains(additional_chars)
                {
                    bump = true;
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
struct CommitSections {
    /// The type of the commit.
    commit_type: String,

    /// The scope of the commit.
    scope: String,

    /// Additional characters in the commit type. For example, in the commit `feat!: Add new feature`, the char `!`
    /// would be in thjis field.
    commit_type_additional_chars: String,

    /// The message of the commit.
    _message: String,
}

/// Extracts the commit sections from a commit message.
///
/// # Arguments
///
/// * `commit` - Commit message that will be parsed.
///
fn extract_commit_sections(commit: &str) -> CommitSections {
    let mut commit_type = String::new();
    let mut scope = String::new();
    let mut commit_type_additional_chars = String::new();
    let mut message = String::new();

    let mut commit_type_done = false;
    let mut scope_done = false;
    let mut commit_type_additional_chars_done = false;
    let mut scope_found = false;

    for c in commit.chars() {
        if !commit_type_done {
            match c {
                '(' => {
                    scope_found = true;
                    commit_type_done = true;
                }
                ':' => {
                    commit_type_done = true;
                    scope_done = true;
                    commit_type_additional_chars_done = true;
                }
                _ => commit_type.push(c),
            }
            continue;
        }

        if !scope_done && scope_found {
            match c {
                ')' => scope_done = true,
                _ => scope.push(c),
            }
            continue;
        }

        if !commit_type_additional_chars_done {
            match c {
                ':' => commit_type_additional_chars_done = true,
                _ => commit_type_additional_chars.push(c),
            }
            continue;
        }

        message.push(c);
    }

    CommitSections {
        commit_type: commit_type.trim().to_string(),
        scope: scope.trim().to_string(),
        commit_type_additional_chars: commit_type_additional_chars.trim().to_string(),
        _message: message.trim().to_string(),
    }
}

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

pub fn calculate_bump(mut version: &Version, rules: &Vec<BumpRule>, commits: &Vec<String>) {
    for commit in commits {
        // for rule in rules {}
        println!("{:?}", extract_commit_sections(commit));
    }
}

#[derive(Debug)]

struct CommitSections {
    commit_type: String,
    scope: String,
    commit_type_additional_chars: String,
    message: String,
}

fn extract_commit_sections(commit: &String) -> CommitSections {
    let mut commit_type = String::new();
    let mut scope = String::new();
    let mut commit_type_additional_chars = String::new();
    let mut message = String::new();

    let mut commit_type_done = false;
    let mut scope_done = false;
    let mut commit_type_additional_chars_done = false;

    for c in commit.chars() {
        if !commit_type_done {
            if c == '(' {
                commit_type_done = true;
            } else {
                commit_type.push(c);
            }
        } else if !scope_done {
            if c == ')' {
                scope_done = true;
            } else {
                scope.push(c);
            }
        } else if !commit_type_additional_chars_done {
            if c == ':' {
                commit_type_additional_chars_done = true;
            } else {
                commit_type_additional_chars.push(c);
            }
        } else {
            message.push(c);
        }
    }

    CommitSections {
        commit_type: commit_type.trim().to_string(),
        scope: scope.trim().to_string(),
        commit_type_additional_chars: commit_type_additional_chars.trim().to_string(),
        message: message.trim().to_string(),
    }
}

//! Module containing version related help functions and types to define version increments.
//!
//! Version increments follow the Semantic Versioning 2.0
//!

use semver::{BuildMetadata, Prerelease, Version};

/// Types for different version.
/// The increment types follow the Semantic Version specification.
#[derive(Eq, PartialEq, Hash)]
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

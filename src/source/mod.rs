//! This module includes the different kinds of sources that can be used
//! to get the required data to calculate a version bump.
//!
//! List of sources:
//! - `git`: uses the `git` command and git history as the source of truth.
//! - `github`: uses the GitHub REST API as the source of truth.
//!

use crate::{
    error::Error,
    git::{Commit, Tag},
};
use enum_dispatch::enum_dispatch;

// pub mod git;
pub mod github;

/// Trait to describe all common actions that all sources need to implement.
#[enum_dispatch]
pub trait SourceActions<'a> {
    /// Returns an Iterator that will return commits and their associated tags for version bump. This iterator may skipped not
    /// required commits or tags which are not required to calculate the version bump.
    ///
    /// # Arguments
    ///
    /// * `sha` - The commit sha to start the iteration from.
    ///
    /// # Errors
    ///
    /// Check each source implementation to check specific source errors.
    ///
    fn get_ref_iterator(
        &self,
        sha: &'a str,
    ) -> Result<Box<dyn Iterator<Item = Result<Reference, Error>> + '_>, Error>;

    /// Returns the latest commit sha.
    fn get_latest_commit_sha(&self) -> Result<String, Error>;
}

/// Type used to wrap obtained references from iterating over commits.
pub struct Reference {
    /// Commit associated with the reference.
    pub commit: Option<Commit>,
    /// Tags associated with the reference.
    pub tags: Option<Vec<Tag>>,
}

/// Type used to wrap different source kinds.
///
/// This type uses the `enum_dispatch` macro to automatically implement the `SourceActions` trait, avoiding
/// to manually implementing the trait with a match statement calling the function depending on the source kind.
///
#[enum_dispatch(SourceActions)]
pub enum SourceKind<'a> {
    // TODO: Implement git source
    // Git(git::GitSource),
    Github(github::GithubSource<'a>),
}

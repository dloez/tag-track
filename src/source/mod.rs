//! This module includes the different kinds of sources that can be used
//! to get the required data to calculate a version bump.
//!
//! List of sources:
//! - `git`: uses the `git` command and git history as the source of truth.
//! - `github`: uses the GitHub REST API as the source of truth.
//!

use crate::error::Error;
use crate::git::Tag;
use enum_dispatch::enum_dispatch;

// pub mod git;
pub mod github;

/// Trait to describe all common actions that all sources need to implement.
#[enum_dispatch]
pub trait SourceActions<'a> {
    /// Returns commit messages. This function can not be called without `fetch_from_commit` being called before
    /// as that is the function that feeds tag track with data from the source.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with the type of `error::ErrorKind::SourceNotFetched` if the function is being
    /// called without calling `fetch_from_commit` before.
    ///
    fn get_commits(&'a mut self, sha: &'a str) -> Result<github::CommitIterator, Error>;

    /// Returns the oldest closest git tag. This function can not be called without `fetch_from_commit` being called before
    /// as that is the function that feeds tag track with data from the source.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with the type of `error::ErrorKind::SourceNotFetched` if the function is being
    /// called without calling `fetch_from_commit` before.
    ///
    fn get_closest_tags(&self) -> Result<&Vec<Tag>, Error>;
}

/// Type used to wrap different source kinds.
///
/// This type uses the `enum_dispatch` macro to automatically implement the `SourceActions` trait, avoiding
/// to manually implementing the trait with a match statement calling the function depending on the source kind.
///
#[enum_dispatch(SourceActions)]
pub enum SourceKind<'a> {
    // Git(git::GitSource),
    Github(github::GithubSource<'a>),
}

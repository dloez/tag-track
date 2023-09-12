//! This module includes the different kinds of sources that can be used
//! to get the required data to calculate a version bump.
//!
//! List of sources:
//! - `git`: uses the `git` command and git history as the source of truth.
//! - `github`: uses the GitHub REST API as the source of truth.
//!

use crate::error::Error;
use enum_dispatch::enum_dispatch;

pub mod git;
pub mod github;

/// Trait to describe all common actions that all sources need to implement.
#[enum_dispatch]
pub trait SourceActions {
    /// Returns commit messages. This function can not be called without `fetch_from_commit` being called before
    /// as that is the function that feeds tag track with data from the source.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with the type of `error::ErrorKind::SourceNotFetched` if the function is being
    /// called without calling `fetch_from_commit` before.
    ///
    fn get_commit_messages(&self) -> Result<&Vec<String>, Error>;

    /// Returns the oldest closest git tag. This function can not be called without `fetch_from_commit` being called before
    /// as that is the function that feeds tag track with data from the source.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with the type of `error::ErrorKind::SourceNotFetched` if the function is being
    /// called without calling `fetch_from_commit` before.
    ///
    fn get_closest_oldest_tag(&self) -> Result<&String, Error>;

    /// Fetches the source to gather the required data to calculate a version bump, this data can be obtained differently depending
    /// on the source kind.
    ///
    /// # Arguments
    ///
    /// * `sha` - git commit SHA from where the commits should be obtained. This SHA typically corresponds to the current
    /// HEAD commit.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with specific kinds depending of the source if there was an error while fetching the data.
    ///
    fn fetch_from_commit(&mut self, sha: &str) -> Result<(), Error>;
}

/// Type used to wrap different source kinds.
///
/// This type uses the `enum_dispatch` macro to automatically implement the `SourceActions` trait, avoiding
/// to manually implementing the trait with a match statement calling the function depending on the source kind.
///
#[enum_dispatch(SourceActions)]
pub enum SourceKind {
    Git(git::GitSource),
    Github(github::GithubSource),
}

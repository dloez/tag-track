//! This module includes the git source. The git source uses the the system
//! `git` installation by using the `git` module and reads the local git history to
//! fetch the required data.
//!
//! This source is useful for local development.
//!

use crate::error::{Error, ErrorKind};
use crate::git;
use crate::source::SourceActions;

/// Type that represents the required data for `tag-track` to calculate a version bump.
pub struct GitSource {
    /// Commit messages used to calculate the version bump.
    commit_messages: Vec<String>,
    /// Oldest closest git tag. All commits between the referenced commit by this tag
    /// and the current/given commit will be used to calculate the version bump.
    oldest_closest_tag: String,
    /// Commit SHA referenced by the tag `oldest_closest_tag`.
    oldest_closest_tag_commit_sha: String,
    /// `true` if the source has been fetched by calling `fetch_from_commit`, `false` other wise
    /// If it is `false`, getter functions will return `error::SourceNotFetched`.
    remote_fetched: bool,
}

impl GitSource {
    /// Returns a new instance of a `GitSource` source.
    pub fn new() -> Self {
        Self {
            commit_messages: vec![],
            oldest_closest_tag: "".to_owned(),
            oldest_closest_tag_commit_sha: "".to_owned(),
            remote_fetched: false,
        }
    }
}

impl SourceActions for GitSource {
    /// Returns commit messages. This function can not be called without `fetch_from_commit` being called before
    /// as that is the function that feeds tag track with data from the source.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with the type of `error::ErrorKind::SourceNotFetched` if the function is being
    /// called without calling `fetch_from_commit` before.
    ///
    fn get_commit_messages(&self) -> Result<&Vec<String>, Error> {
        if !self.remote_fetched {
            return Err(Error::new(ErrorKind::SourceNotFetched, None));
        }
        Ok(&self.commit_messages)
    }

    /// Fetches the source to gather the required data to calculate a version bump. This source uses the git history to
    /// gather the data.
    ///
    /// # Arguments
    ///
    /// * `sha` - git commit SHA from where the commits should be obtained. This SHA typically corresponds to the current
    /// HEAD commit.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if there was an unexpected error
    /// while calling a command.
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::Other` if the command called returned an unexpected error
    /// causing that the tag commit SHA cannot be obtained.
    ///
    fn fetch_from_commit(&mut self, sha: &str) -> Result<(), Error> {
        self.oldest_closest_tag = match git::get_oldest_closest_tag(Some(sha)) {
            Ok(tag) => tag,
            Err(error) => return Err(error),
        };

        self.oldest_closest_tag_commit_sha = match git::get_tag_commit_sha(&self.oldest_closest_tag)
        {
            Ok(tag_commit_sha) => tag_commit_sha,
            Err(error) => return Err(error),
        };

        self.commit_messages = git::get_commit_messages(&self.oldest_closest_tag_commit_sha, sha)?;
        self.remote_fetched = true;
        Ok(())
    }

    /// Returns the oldest closest git tag. This function can not be called without `fetch_from_commit` being called before
    /// as that is the function that feeds tag track with data from the source.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with the type of `error::ErrorKind::SourceNotFetched` if the function is being
    /// called without calling `fetch_from_commit` before.
    ///
    fn get_closest_oldest_tag(&self) -> Result<&String, Error> {
        if !self.remote_fetched {
            return Err(Error::new(ErrorKind::SourceNotFetched, None));
        }
        Ok(&self.oldest_closest_tag)
    }
}

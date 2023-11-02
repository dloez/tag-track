//! Module containing git utilities to work with the system git installation and git history.
//!
//! To spawn shell commands it is being used the function `std::process::Command::new` so git
//! must be installed and in the path.
//!

use std::process::Command;

use crate::error::{Error, ErrorKind};
use crate::parsing::{CommitDetails, TagDetails};

/// Type to define a Git commit.
#[derive(Debug)]
pub struct Commit {
    /// Commit SHA.
    pub sha: String,

    /// Commit message.
    pub message: String,

    /// Commit details such as fields from conventional commits.
    pub details: Option<CommitDetails>,
}

/// Type to define a Git tag.
#[derive(Debug, Clone)]
pub struct Tag {
    /// Commit SHA referenced by tag.
    pub commit_sha: String,

    /// Tag name.
    pub name: String,

    /// Tag details such as version and scope.
    pub details: Option<TagDetails>,
}

/// Verifies the git installation and if the command is being spawned inside a git working tree.
/// In case git is available and it is being called inside a git working tree, the function will return
/// `Ok(())`.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::MissingGit` if the git command was not available.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if there was an unexpected error
/// while calling a command.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::NoGitWorkingTree` if git is being called outside a
/// git working tree.
///
pub fn verify_git() -> Result<(), Error> {
    if let Err(error) = Command::new("git").arg("--version").output() {
        return Err(Error::new(ErrorKind::MissingGit, Some(&error.to_string())));
    }

    let output_result = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::GenericCommandFailed,
                Some(&error.to_string()),
            ))
        }
    };

    if !output.status.success() {
        return Err(Error::new(ErrorKind::NotGitWorkingTree, None));
    }

    Ok(())
}

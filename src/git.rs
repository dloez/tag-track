//! Module containing git utilities to work with the system git installation and git history.
//!
//! To spawn shell commands it is being used the function `std::process::Command::new` so git
//! must be installed and in the path.
//!

use std::process::Command;

use crate::error::{Error, ErrorKind};

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

/// Returns the current commit SHA or an `error::Error` if there was an error while trying to get the commit SHA.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if there was an unexpected error
/// while calling a command.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::Other` if the command called returned an unexpected error
/// causing that the current commit SHA cannot be obtained.
///
pub fn get_current_commit_sha() -> Result<String, Error> {
    let output_result = Command::new("git").arg("rev-parse").arg("HEAD").output();

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
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get current commit, error code: \"{}\", stderr: \"{}\"",
                output.status.code().unwrap(),
                stderr.trim(),
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

/// Returns the latest closest git tag name from the given commit.
///
/// In the following tree, the tag 0.1.0 will be returned:
/// ```
///  tag 0.2.0
///      |
/// given commit
///      |
///   commit
///      |
///  tag 0.1.0
/// ```
///
/// # Arguments
///
/// * `from_commit` - From which commit the oldest closest tag will be optained. If `None` is given, it will default to `HEAD`.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if there was an unexpected error
/// while calling a command.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::Other` if the command called returned an unexpected error
/// causing that the oldest closest tag cannot be obtained.
///
pub fn get_oldest_closest_tag(from_commit: Option<&str>) -> Result<String, Error> {
    let mut binding = Command::new("git");
    let command = binding.arg("describe").arg("--abbrev=0").arg("--tags");

    let output_result = match from_commit {
        Some(from_commit) => {
            command.arg(from_commit);
            command.output()
        }
        None => command.output(),
    };

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
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get closest tag, error code: \"{}\", stderr: \"{}\"",
                output.status.code().unwrap(),
                stderr.trim(),
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

/// From a given tag name return its corresponding commit SHA.
///
/// # Arguments
///
/// * `tag` - Git tag name.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if there was an unexpected error
/// while calling a command.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::Other` if the command called returned an unexpected error
/// causing that the tag commit SHA cannot be obtained.
///
pub fn get_tag_commit_sha(tag: &str) -> Result<String, Error> {
    let output_result = Command::new("git")
        .arg("rev-list")
        .args(["-n", "1"])
        .arg(tag)
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
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get tag commit sha, error code: \"{}\", stderr: \"{}\"",
                output.status.code().unwrap(),
                stderr.trim(),
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

/// Return all commit messages between the given `from_commit` SHA and until the `until_commit` SHA where the oldest
/// commit must be the `from_commit` commit.
///
/// # Arguments
///
/// * `from_commit` - commit SHA of the commit from where the messages should be obtained. The given commit message will
/// not be included. This commit should be temporarily older than the commit given in the `until_commit` argument.
///
/// * `until_commit` - commit SHA of the commit until where the messages should be obtained. The given commit message
/// will be included. This commit should be temporarily newer than the commit given in the `from_commit` argument.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if there was an unexpected error
/// while calling a command.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::Other` if the command called returned an unexpected error
/// causing that the tag commit SHA cannot be obtained.
///
pub fn get_commit_messages(from_commit: &str, until_commit: &str) -> Result<Vec<String>, Error> {
    let output_result = Command::new("git")
        .arg("log")
        .arg("--format=%s")
        .arg("--ancestry-path")
        .arg(format!("{}..{}", from_commit, until_commit))
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
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get commit between '{}' and '{}', error code: \"{}\", stderr: \"{}\"",
                from_commit,
                until_commit,
                output.status.code().unwrap(),
                stderr.trim(),
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout.lines().map(|s| s.to_owned()).collect())
}

/// Create the given annotated tag with the given message. Returns `Ok(())` if no errors were found.
///
/// # Arguments
///
/// * `tag` - Git tag name.
///
/// * `tag_message` - Git tag message.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if there was an unexpected error
/// while calling a command.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::Other` if the command called returned an unexpected error
/// causing that the git tag could not be created.
///
pub fn create_tag(tag: &str, tag_message: &str) -> Result<(), Error> {
    let output_result = Command::new("git")
        .arg("tag")
        .args(["-a", tag])
        .args(["-m", tag_message])
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
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not create tag '{}', error code: \"{}\", stderr: \"{}\"",
                tag,
                output.status.code().unwrap(),
                stderr.trim(),
            )),
        ));
    }

    Ok(())
}

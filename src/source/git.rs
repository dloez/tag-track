//! This module includes the git source. The git source uses the the system
//! `git` installation by using the `git` module and reads the local git history to
//! fetch the required data.
//!
//! This source is useful for local development.
//!

use std::process::Command;
use std::vec;

use crate::config::Config;
use crate::error::{Error, ErrorKind};
use crate::git::{Commit, Tag};
use crate::parsing::{parse_commit_details, parse_tag_details};
use crate::source::{Reference, SourceActions};

/// Type that represents the Git as a source.
pub struct GitSource<'a> {
    /// Tag Track configuration.
    config: &'a Config,
}

impl<'a> GitSource<'a> {
    /// Returns a new instance of a `GitSource` source.
    ///
    /// # Arguments
    ///
    /// * `config` - Tag Track configuration.
    ///
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

/// Trait to describe all common actions that all sources need to implement.
impl<'a> SourceActions<'a> for GitSource<'a> {
    /// Returns an Iterator that will return commits and their associated tags for version bump. This iterator may skipped not
    /// required commits or tags which are not required to calculate the version bump.
    ///
    /// # Arguments
    ///
    /// * `sha` - The commit sha to start the iteration from.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::MissingGitTags` if there are no tags in the source.
    ///
    fn get_ref_iterator(
        &self,
        sha: &'a str,
    ) -> Result<Box<dyn Iterator<Item = Result<Reference, Error>> + '_>, Error> {
        let tags = get_all_tags(&self.config.tag_pattern)?;
        if tags.is_none() {
            return Err(Error::new(
                ErrorKind::MissingGitTags,
                Some("no tags found for repository"),
            ));
        }

        Ok(Box::new(RefIterator::new(sha, tags.unwrap(), self.config)))
    }

    fn get_latest_commit_sha(&self) -> Result<String, Error> {
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

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    }

    // pub fn create_tag(tag: &str, tag_message: &str) -> Result<(), Error> {
    //     let output_result = Command::new("git")
    //         .arg("tag")
    //         .args(["-a", tag])
    //         .args(["-m", tag_message])
    //         .output();

    //     let output = match output_result {
    //         Ok(output) => output,
    //         Err(error) => {
    //             return Err(Error::new(
    //                 ErrorKind::GenericCommandFailed,
    //                 Some(&error.to_string()),
    //             ))
    //         }
    //     };

    //     if !output.status.success() {
    //         let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    //         return Err(Error::new(
    //             ErrorKind::Other,
    //             Some(&format!(
    //                 "can not create tag '{}', error code: \"{}\", stderr: \"{}\"",
    //                 tag,
    //                 output.status.code().unwrap(),
    //                 stderr.trim(),
    //             )),
    //         ));
    //     }

    //     Ok(())
    // }
}

/// Type used to iterate over GitHub references on the repository history.
/// This type implements the `Iterator` trait and performs paginated requests to the GitHub REST API.
pub struct RefIterator<'a> {
    /// List of version scopes that have not been found yet in the commits.
    version_scopes: Vec<String>,
    /// Current commit index.
    current_elem: u64,
    /// If the iterator has finished iterating over the commits.
    is_finished: bool,

    /// Commit SHA from where the iteration will start.
    sha: &'a str,
    /// List of tags obtained from the GitHub REST API.
    tags: Vec<Tag>,
    /// Tag Track configuration.
    config: &'a Config,
}

impl<'a> RefIterator<'a> {
    /// Returns a new instance of a `CommitIterator`.
    fn new(sha: &'a str, tags: Vec<Tag>, config: &'a Config) -> Self {
        RefIterator {
            version_scopes: config.version_scopes.clone(),
            is_finished: false,
            current_elem: 0,

            sha,
            tags,
            config,
        }
    }
}

impl<'a> Iterator for RefIterator<'a> {
    type Item = Result<Reference, Error>;

    /// Returns the next commit and its associated tags until the required commits to calculate the version bump have
    /// been returned. If using scoped versioning, commits with scopes which tag has been already returned will be skipped.
    ///
    /// If a tag is associated with multiple commits, the tag with the biggest version will be returned. This is also true
    /// if scoped versioning is used and there are multiple tags with the same scope in the same commit.
    ///
    /// If there is a commit that does not conform the given commit pattern, it will be returned with `None` in the details
    /// field. If there is a tag that does not conform the given tag pattern, it will be skipped.
    ///
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }

        let commit = match get_n_commit_from_commit_sha(
            self.current_elem,
            self.sha,
            &self.config.commit_pattern,
        ) {
            Ok(commit) => commit,
            Err(error) => return Some(Err(error)),
        };
        self.current_elem += 1;
        if commit.is_none() {
            self.is_finished = true;
            return None;
        }
        let commit = commit.unwrap();

        let tags = match find_tags_from_commit_sha(&commit.sha, &self.tags, &self.version_scopes) {
            Ok(tags) => tags,
            Err(error) => return Some(Err(error)),
        };

        if tags.is_some() {
            for tag in tags.as_ref().unwrap() {
                let tag_details = match &tag.details {
                    Some(details) => details,
                    None => continue,
                };
                self.version_scopes
                    .retain(|scope| scope != tag_details.scope.as_ref().unwrap_or(&String::new()));
            }

            if self.version_scopes.is_empty() {
                self.is_finished = true;
            }
        }

        let commit_details = match &commit.details {
            Some(details) => details,
            None => {
                return Some(Ok(Reference {
                    commit: Some(commit),
                    tags,
                }))
            }
        };

        if self
            .version_scopes
            .contains(commit_details.scope.as_ref().unwrap_or(&String::new()))
        {
            return Some(Ok(Reference {
                commit: Some(commit),
                tags,
            }));
        }

        if tags.is_none() {
            return self.next();
        }

        Some(Ok(Reference { commit: None, tags }))
    }
}

/// Obtains all tags using the Git CLI.
///
/// # Arguments
///
/// * `repo_id` - GitHub repository identifier that will be used to query commits.
///
/// * `api_url` - GitHub REST API base URL.
///
/// * `token` - GitHub REST API authentication token. If it is `None`, requests will not be authenticated, if it has
/// a value, requests will be authenticated.
///
fn get_all_tags(tag_pattern: &str) -> Result<Option<Vec<Tag>>, Error> {
    // create code for a function that returns a vector of Tags obtained using the git CLI
    // use the tag_pattern argument to parse the tag details
    // use the Tag struct to represent the tag details
    // return the vector
    let output_result = Command::new("git")
        .arg("show-ref")
        .arg("--tags")
        .arg("-d")
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
                "can not get tags, error code: \"{}\", stderr: \"{}\"",
                output.status.code().unwrap(),
                stderr.trim(),
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Ok(None);
    }

    let mut tags: Vec<Tag> = vec![];
    for (i, line) in stdout.split('\n').enumerate() {
        if i % 2 == 0 {
            continue;
        }

        let line = line.replace("refs/tags/", "").replace("^{}", "");
        let mut sha = String::new();
        let mut name = String::new();
        let mut sha_done = false;
        for c in line.chars() {
            if !sha_done {
                match c {
                    ' ' => sha_done = true,
                    _ => sha.push(c),
                }
                continue;
            }
            name.push(c);
        }

        tags.push(Tag {
            details: parse_tag_details(&name, tag_pattern)?,
            name,
            commit_sha: sha,
        });
    }

    if tags.is_empty() {
        return Ok(None);
    }

    Ok(Some(tags))
}

/// Obtains all commits from a given commit SHA using the Git CLI.
///
/// # Arguments
///
/// * `n`: Which commit should be returned where `0` is the latest commit.
///
/// * `commit_pattern`: Pattern used to extract the commit details.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GenericCommandFailed` if the `git` command fails.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::Other` if the command output cannot be converted to a utf8 string.
///
/// Returns `error::Error` with a kind of `error::ErrorKind::InvalidRegexPattern` if the commit pattern is invalid.
///
fn get_n_commit_from_commit_sha(
    n: u64,
    commit_sha: &str,
    commit_pattern: &str,
) -> Result<Option<Commit>, Error> {
    let output_result = Command::new("git")
        .arg("rev-list")
        .arg(commit_sha)
        .arg("--max-count=1")
        .arg(format!("--skip={}", n))
        .arg("--format=%H %s")
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
                "can not get current commit, error code: \"{}\", stderr: \"{}\"",
                output.status.code().unwrap(),
                stderr.trim(),
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Ok(None);
    }

    let mut sha = String::new();
    let mut message = String::new();
    let mut sha_done = false;
    let mut cleaned = false;
    for c in stdout.chars() {
        if !cleaned {
            if c == '\n' {
                cleaned = true;
            }
            continue;
        }

        if !sha_done {
            match c {
                ' ' => sha_done = true,
                _ => sha.push(c),
            }
            continue;
        }
        message.push(c);
    }
    Ok(Some(Commit {
        sha,
        details: parse_commit_details(&message, commit_pattern)?,
        message,
    }))
}

/// From a given list of `GitHub` tag, find the list of tags referencing a commit SHA equal to the given `sha` argument.
/// If a tag with the given SHA cannot be found, `None` will be returned. If there are multiple tags referencing the same
/// commit SHA, the tag with the highest version will be returned. This is also true if scoped versioning is used and there
/// are multiple tags with the same scope in the same commit.
///
/// # Arguments
///
/// * `sha` - Commit SHA that will be searched inside the tags commits.
///
/// * `tags` - List of tags.
///
/// * `tag_pattern` - Pattern used to extract the tag details.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::TagPatternError` if the tag pattern is invalid.
///
fn find_tags_from_commit_sha(
    sha: &str,
    tags: &[Tag],
    valid_scopes: &[String],
) -> Result<Option<Vec<Tag>>, Error> {
    let mut found_tags: Vec<Tag> = vec![];
    for tag in tags {
        if tag.commit_sha != sha {
            continue;
        }

        let tag_details = match &tag.details {
            Some(details) => details,
            None => continue,
        };

        if !valid_scopes.contains(tag_details.scope.as_ref().unwrap_or(&String::new())) {
            continue;
        }

        if found_tags.is_empty() {
            found_tags.push(tag.clone());
            continue;
        }

        let mut found = false;
        for found_tag in &mut found_tags {
            let found_tag_details = match &found_tag.details {
                Some(details) => details,
                None => continue,
            };
            if found_tag_details.scope.as_ref().unwrap_or(&String::new())
                == tag_details.scope.as_ref().unwrap_or(&String::new())
                && tag_details.version > found_tag_details.version
            {
                *found_tag = tag.clone();
                found = true;
                break;
            }
        }

        if !found {
            found_tags.push(tag.clone());
        }
    }

    if found_tags.is_empty() {
        return Ok(None);
    }

    Ok(Some(found_tags))
}

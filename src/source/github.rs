//! This module includes the GitHub source. The GitHub source uses the GitHub REST API
//! to fetch the required data.
//!
//! This source is useful for working in CI environments, where the git history is neither not available
//! or partially available.
//!

use std::vec;

use crate::config::Config;
use crate::error::{Error, ErrorKind};
use crate::git::{Commit, Tag};
use crate::parsing::{parse_commit_details, parse_tag_details};
use crate::source::SourceActions;
use reqwest;
use serde::Deserialize;

/// GitHub REST API base URL.
pub const GITHUB_API_BASE_URL: &str = "https://api.github.com";
/// GitHub REST API URI for querying tags. Must be used in combination with `GITHUB_BASE_URI`.
const GITHUB_TAGS_URI: &str = "/tags";
/// GitHub REST API URI for querying commits. Must be used in combination with `GITHUB_BASE_URI`.
const GITHUB_COMMITS_URI: &str = "/commits";
/// Content for the `User-Agent` header.
const USER_AGENT: &str = "tag-track";
/// Name for the authorization header for authorizing GitHub REST API requests.
const AUTH_HEADER: &str = "authorization";

/// Default elements per page used for paginated requests.
const DEFAULT_PER_PAGE: u64 = 100;

/// Type that represents the required data for `tag-track` to calculate a version bump.
pub struct GithubSource<'a> {
    /// Tag Track configuration.
    config: &'a Config,

    /// GitHub repository identifier `org/repo-name`, example `dloez/tag-track`.
    repo_id: String,
    /// GitHub REST API base URL.
    api_url: String,
    /// GitHub REST API authentication token to authorize requests.
    token: Option<String>,
}

impl<'a> GithubSource<'a> {
    /// Returns a new instance of a `GitHubSource` source.
    ///
    /// # Arguments
    ///
    /// * `repo_id` - GitHub repository identifier in the format `org/repo-name`, example `dloez/tag-track`.
    ///
    /// * `api_url` - GitHub REST API base URL.
    ///
    /// * `token` - GitHub REST API authentication token to authorize requests.
    ///
    /// * `config` - Tag Track configuration.
    ///
    pub fn new(
        config: &'a Config,
        repo_id: String,
        api_url: String,
        token: Option<String>,
    ) -> Self {
        Self {
            config,
            repo_id,
            api_url,
            token,
        }
    }
}

/// Trait to describe all common actions that all sources need to implement.
impl<'a> SourceActions<'a> for GithubSource<'a> {
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
    fn get_commits(&self, sha: &'a str) -> Result<CommitIterator, Error> {
        let tags = get_all_tags(&self.repo_id, &self.api_url, &self.token)?;
        if tags.is_empty() {
            return Err(Error::new(
                ErrorKind::MissingGitTags,
                Some("no tags found for repository"),
            ));
        }

        Ok(CommitIterator::new(
            sha,
            tags,
            &self.repo_id,
            &self.api_url,
            &self.token,
            self.config,
        ))
    }
}

/// Used to deserialize responses from `https://api.github.com/repos/org/repo_name/tags`.
/// Only the required fields by `tag-track` are included.
#[derive(Debug, Deserialize, Clone)]
struct GithubTag {
    name: String,
    commit: GithubTagCommit,
}

/// Used to deserialize responses from `https://api.github.com/repos/org/repo_name/tags`.
/// Only the required fields by `tag-track` are included.
#[derive(Debug, Deserialize, Clone)]
struct GithubTagCommit {
    sha: String,
}

impl GithubTag {
    /// Converts a `GithubTag` into a `Tag`. If the tag details cannot be extracted,
    /// the `details` struct will be `None`.
    ///
    /// # Arguments
    ///
    /// * `tag_pattern` - Pattern used to extract the tag details.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::TagPatternError` if the tag pattern is invalid.
    ///
    fn to_git_tag(self, tag_pattern: &str) -> Result<Tag, Error> {
        let tag_details = parse_tag_details(&self.name, tag_pattern)?;

        Ok(Tag {
            name: self.name,
            commit_sha: self.commit.sha,
            details: tag_details,
        })
    }
}

/// Used to deserialize responses from `https://api.github.com/repos/org/repo_name/commits`.
/// Only the required fields by `tag-track` are included.
#[derive(Debug, Deserialize, Clone)]
struct GithubCommitDetails {
    sha: String,
    commit: GithubCommit,
}

/// Used to deserialize responses from `https://api.github.com/repos/org/repo_name/commits`.
/// Only the required fields by `tag-track` are included.
#[derive(Debug, Deserialize, Clone)]
struct GithubCommit {
    message: String,
}

impl GithubCommitDetails {
    /// Converts a `GithubCommitDetails` into a `Commit`. If the commit details cannot be extracted,
    /// the `details` struct will be `None`.
    ///
    /// # Arguments
    ///
    /// * `commit_pattern` - Pattern used to extract the commit details.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::CommitPatternError` if the commit pattern is invalid.
    ///
    fn to_git_commit(self, commit_pattern: &str) -> Result<Commit, Error> {
        let commit_details = parse_commit_details(&self.commit.message, commit_pattern)?;

        Ok(Commit {
            sha: self.sha,
            message: self.commit.message,
            details: commit_details,
        })
    }
}

/// Type used to iterate over GitHub commits. This type implements the `Iterator` trait
/// and performs paginated requests to the GitHub REST API.
pub struct CommitIterator<'a> {
    /// List of commits obtained from the GitHub REST API. Commits are obtained on batches of 100 elements.
    commits: Vec<GithubCommitDetails>,
    /// List of version scopes that have not been found yet in the commits.
    version_scopes: Vec<String>,
    /// Current GitHub REST API page number.
    page: u64,
    /// Elements per page used for paginated requests.
    per_page: u64,
    /// If the iterator has finished iterating over the commits.
    is_finished: bool,
    /// Current element index in the `commits` vector.
    current_elem: u64,

    /// Commit SHA from where the iteration will start.
    sha: &'a str,
    /// List of tags obtained from the GitHub REST API.
    tags: Vec<GithubTag>,
    /// GitHub repository identifier `org/repo-name`, example `dloez/tag-track`.
    repo_id: &'a String,
    /// GitHub REST API base URL.
    api_url: &'a String,
    /// GitHub REST API authentication token to authorize requests.
    github_token: &'a Option<String>,
    /// Tag Track configuration.
    config: &'a Config,
}

impl<'a> CommitIterator<'a> {
    /// Returns a new instance of a `CommitIterator`.
    fn new(
        sha: &'a str,
        tags: Vec<GithubTag>,
        repo_id: &'a String,
        api_url: &'a String,
        github_token: &'a Option<String>,
        config: &'a Config,
    ) -> Self {
        CommitIterator {
            commits: vec![],
            version_scopes: config.version_scopes.clone(),
            page: 0,
            per_page: DEFAULT_PER_PAGE,
            is_finished: false,
            current_elem: 0,

            sha,
            tags,
            repo_id,
            api_url,
            github_token,
            config,
        }
    }
}

impl<'a> Iterator for CommitIterator<'a> {
    type Item = Result<(Commit, Option<Vec<Tag>>), Error>;

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

        if self.current_elem == 0 {
            self.commits = match get_commits_from_commit_sha(
                &self.repo_id,
                &self.api_url,
                self.sha,
                &self.github_token,
                &self.page,
                &self.per_page,
            ) {
                Ok(commits) => commits,
                Err(error) => {
                    return Some(Err(error));
                }
            };
        };

        let commit = self.commits.get(self.current_elem as usize);
        self.current_elem += 1;
        if let None = commit {
            return None;
        }

        let commit: Commit = match commit
            .unwrap()
            .clone()
            .to_git_commit(&self.config.commit_pattern)
        {
            Ok(commit) => commit,
            Err(error) => return Some(Err(error)),
        };
        let tags =
            match find_tags_from_commit_sha(&commit.sha, &self.tags, &self.config.tag_pattern) {
                Ok(tags) => tags,
                Err(error) => return Some(Err(error)),
            };

        let commit_details = match &commit.details {
            Some(details) => details,
            None => {
                if tags.is_empty() {
                    return Some(Ok((commit, None)));
                }

                return Some(Ok((commit, Some(tags))));
            }
        };

        if tags.is_empty()
            && self
                .version_scopes
                .contains(commit_details.scope.as_ref().unwrap_or(&String::new()))
        {
            return Some(Ok((commit, None)));
        }

        if tags.is_empty()
            && !self
                .version_scopes
                .contains(commit_details.scope.as_ref().unwrap_or(&String::new()))
        {
            return self.next();
        }

        for tag in &tags {
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

        Some(Ok((commit, Some(tags))))
    }
}

/// Obtains tags from the given repository. If `token` is given, the requests will be authorized.
/// The requests performed by this function are not yet paginated.
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
/// * `page` - GitHub REST API requests page number. This number must not exceed `u64` limits.
///
/// * `per_page` - GitHub REST API elements per request page. Limit is `100`.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GitHubRestError` if there was an unexpected response
/// from the GitHub REST API.
///
fn get_tags(
    repo_id: &String,
    api_url: &String,
    token: &Option<String>,
    page: &u64,
    per_page: &u64,
) -> Result<Vec<GithubTag>, Error> {
    let client = reqwest::blocking::Client::new();
    let mut client = client
        .get(format!(
            "{}/repos/{}{}?page={}&per_page={}",
            api_url, repo_id, GITHUB_TAGS_URI, page, per_page
        ))
        .header(reqwest::header::USER_AGENT, USER_AGENT);

    if let Some(token) = token {
        client = client.header(AUTH_HEADER, format!("Bearer {}", token));
    }

    let response = match client.send() {
        Err(error) => {
            return Err(Error::new(
                ErrorKind::GithubRestError,
                Some(&error.to_string()),
            ))
        }
        Ok(res) => res,
    };

    let tags: Vec<GithubTag> = match response.status().is_success() {
        false => {
            return Err(Error::new(
                ErrorKind::GithubRestError,
                Some(&response.text().unwrap()),
            ))
        }
        true => match response.json() {
            Ok(tags) => tags,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::GithubRestError,
                    Some(&error.to_string()),
                ))
            }
        },
    };

    Ok(tags)
}

/// Obtains all tags from the given repository. If `token` is given, the requests will be authorized.
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
fn get_all_tags(
    repo_id: &String,
    api_url: &String,
    token: &Option<String>,
) -> Result<Vec<GithubTag>, Error> {
    let mut page: u64 = 0;
    let mut tags: Vec<GithubTag> = vec![];

    loop {
        let t = get_tags(repo_id, api_url, token, &page, &DEFAULT_PER_PAGE)?;
        if t.is_empty() {
            break;
        }

        tags.reserve(t.len());
        tags.extend(t);
        page += 1;
    }

    Ok(tags)
}

/// Obtains commits from the given `sha` using the GitHub REST API. If `token` is given, the requests will be authorized.
/// Requests to GitHub REST API are paginated.
///
/// # Arguments
///
/// * `repo_id` - GitHub repository identifier that will be used to query commits.
///
/// * `api_url` - GitHub REST API base URL.
///
/// * `sha` - SHA from where the commits will be requested.
///
/// * `token` - GitHub REST API authentication token. If it is `None`, requests will not be authenticated, if it has
/// a value, requests will be authenticated.
///
/// * `page` - GitHub REST API requests page number. This number must not exceed `u64` limits.
///
/// * `per_page` - GitHub REST API elements per request page. Limit is `100`.
///
/// # Errors
///
/// Returns `error::Error` with a kind of `error::ErrorKind::GitHubRestError` if there was an unexpected response
/// from the GitHub REST API.
///
fn get_commits_from_commit_sha(
    repo_id: &String,
    api_url: &String,
    sha: &str,
    token: &Option<String>,
    page: &u64,
    per_page: &u64,
) -> Result<Vec<GithubCommitDetails>, Error> {
    let client = reqwest::blocking::Client::new();
    let mut client = client
        .get(format!(
            "{}/repos/{}{}?sha={}&page={}&per_page={}",
            api_url, repo_id, GITHUB_COMMITS_URI, sha, page, per_page
        ))
        .header(reqwest::header::USER_AGENT, USER_AGENT);

    if let Some(token) = token {
        client = client.header(AUTH_HEADER, format!("Bearer {}", token));
    }

    let response = match client.send() {
        Err(error) => {
            return Err(Error::new(
                ErrorKind::GithubRestError,
                Some(&error.to_string()),
            ))
        }
        Ok(res) => res,
    };

    let commits: Vec<GithubCommitDetails> = match response.status().is_success() {
        false => {
            return Err(Error::new(
                ErrorKind::GithubRestError,
                Some(&response.text().unwrap()),
            ))
        }
        true => match response.json() {
            Ok(commits) => commits,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::GithubRestError,
                    Some(&error.to_string()),
                ))
            }
        },
    };

    Ok(commits)
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
    tags: &[GithubTag],
    tag_pattern: &str,
) -> Result<Vec<Tag>, Error> {
    let mut found_tags: Vec<Tag> = vec![];
    for tag in tags {
        if tag.commit.sha != sha {
            continue;
        }

        let tag = tag.clone().to_git_tag(tag_pattern)?;
        let tag_details = match &tag.details {
            Some(details) => details,
            None => continue,
        };

        if found_tags.is_empty() {
            found_tags.push(tag);
            continue;
        }

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
            }
        }
    }
    Ok(found_tags)
}

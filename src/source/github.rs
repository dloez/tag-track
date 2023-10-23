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
use crate::parsing::{extract_commit_details, extract_tag_details, TagDetails};
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
    /// Closest tags to current or given commit. If scoped versioning is not used, only the closest tag will be stored.
    /// All commits between the including the current one and the one referenced by the oldest tag will be used to calculate
    /// the version bump.
    closest_tags: Vec<TagDetails>,

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
    pub fn new(
        repo_id: String,
        api_url: String,
        token: Option<String>,
        config: &'a Config,
    ) -> Self {
        Self {
            closest_tags: vec![],
            config,
            repo_id,
            api_url,
            token,
        }
    }
}

impl<'a> SourceActions<'a> for GithubSource<'a> {
    fn get_commits(&'a mut self, sha: &'a str) -> Result<self::CommitIterator, Error> {
        let tags = get_all_tags(&self.repo_id, &self.api_url, &self.token)?;
        if tags.is_empty() {
            return Err(Error::new(
                ErrorKind::MissingGitTags,
                Some("no tags found for repository"),
            ));
        }

        Ok(CommitIterator::new(self, sha, tags, &self.config))
    }
}

/// Used to deserialize responses from `https://api.github.com/repos/org/repo_name/tags`.
/// Only the required fields by `tag-track` are included.
#[derive(Debug, Deserialize, Clone)]
struct GithubTag {
    name: String,
    commit: GithubTagCommit,
}

impl From<GithubTag> for Tag {
    fn from(val: GithubTag) -> Self {
        Tag {
            commit_sha: val.commit.sha,
            name: val.name,
        }
    }
}

/// Used to deserialize responses from `https://api.github.com/repos/org/repo_name/tags`.
/// Only the required fields by `tag-track` are included.
#[derive(Debug, Deserialize, Clone)]
struct GithubTagCommit {
    sha: String,
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

impl From<GithubCommitDetails> for Commit {
    fn from(val: GithubCommitDetails) -> Self {
        Commit {
            sha: val.sha,
            message: val.commit.message,
        }
    }
}

/// Type used to iterate over GitHub commits. This type implements the `Iterator` trait
/// and performs paginated requests to the GitHub REST API.
pub struct CommitIterator<'a> {
    page: u64,
    per_page: u64,
    commits: Vec<GithubCommitDetails>,
    source: &'a mut GithubSource<'a>,
    sha: &'a str,
    tags: Vec<GithubTag>,
    config: &'a Config,
    version_scopes: Vec<String>,
    is_finished: bool,
    max_elem: u64,
    current_elem: u64,
}

impl<'a> CommitIterator<'a> {
    /// Returns a new instance of a `CommitIterator`.
    fn new(
        source: &'a mut GithubSource<'a>,
        sha: &'a str,
        tags: Vec<GithubTag>,
        config: &'a Config,
    ) -> Self {
        CommitIterator {
            page: 0,
            per_page: DEFAULT_PER_PAGE,
            commits: vec![],
            source,
            sha,
            tags,
            config,
            version_scopes: config.version_scopes.clone(),
            is_finished: false,
            max_elem: 0,
            current_elem: 0,
        }
    }
}

impl<'a> Iterator for CommitIterator<'a> {
    type Item = Result<Commit, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }

        if self.current_elem == self.max_elem {
            self.commits = match get_commits_from_commit_sha(
                &self.source.repo_id,
                &self.source.api_url,
                self.sha,
                &self.source.token,
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

        let commit: Commit = commit.unwrap().clone().into();
        let tags = match find_tags_from_commit_sha(
            &commit.sha,
            &self.tags,
            &self.config.tag_pattern,
            self.config.version_scopes.is_empty(),
        ) {
            Ok(tags) => tags,
            Err(error) => {
                return Some(Err(error));
            }
        };

        if tags.is_empty() {
            return Some(Ok(commit));
        }

        self.source.closest_tags.extend(tags);
        if self.config.version_scopes.is_empty() {
            self.is_finished = true;
            return Some(Ok(commit));
        }

        let commit_details = match extract_commit_details(&commit, &self.config.commit_pattern) {
            Ok(commit_details) => commit_details,
            Err(error) => {
                return Some(Err(error));
            }
        };
        if let Some(scope) = commit_details.scope.as_ref().map(|s| s.as_str()) {
            self.version_scopes.retain(|x| x != scope);
            return Some(Ok(commit));
        } else {
            return self.next();
        }
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

/// From a given list of `GitHub` tag, find the tag referencing a commit SHA equal to the given `sha` argument.
/// If a tag with the given SHA cannot be found, `None` will be returned.
///
/// # Arguments
///
/// * `sha` - Commit SHA that will be searched inside the tags commits.
///
/// * `tags` - List of tags.
///
fn find_tags_from_commit_sha(
    sha: &str,
    tags: &[GithubTag],
    tag_pattern: &str,
    scoped: bool,
) -> Result<Vec<TagDetails>, Error> {
    let mut found_tags = vec![];
    for tag in tags {
        if tag.commit.sha != sha {
            continue;
        }

        let tag: Tag = tag.clone().into();
        let tag_details = extract_tag_details(&tag, &tag_pattern)?;
        if !scoped {
            found_tags.push(tag_details);
            continue;
        }

        for found_tag in &mut found_tags {
            if found_tag.scope == tag_details.scope && tag_details.version > found_tag.version {
                *found_tag = tag_details.clone();
            }
        }
    }
    Ok(found_tags)
}

//! This module includes the GitHub source. The GitHub source uses the GitHub REST API
//! to fetch the required data.
//!
//! This source is useful for working in CI environments, where the git history is neither not available
//! or partially available.
//!

use crate::error::{Error, ErrorKind};
use crate::source::SourceActions;
use reqwest;
use serde::Deserialize;

/// GitHub REST API base URI.
const GITHUB_BASE_URI: &str = "https://api.github.com/repos";
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
pub struct GithubSource {
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

    /// GitHub repository identifier `org/repo-name`, example `dloez/tag-track`.
    repo_id: String,
    /// GitHub REST API authentication token to authorize requests.
    token: Option<String>,
}

impl GithubSource {
    /// Returns a new instance of a `GitHubSource` source.
    ///
    /// # Arguments
    ///
    /// * `repo_id` - GitHub repository identifier in the format `org/repo-name`, example `dloez/tag-track`.
    ///
    /// * `token` - GitHub REST API authentication token to authorize requests.
    ///
    pub fn new(repo_id: String, token: Option<String>) -> Self {
        Self {
            commit_messages: vec![],
            oldest_closest_tag: "".to_owned(),
            oldest_closest_tag_commit_sha: "".to_owned(),
            remote_fetched: false,
            repo_id,
            token,
        }
    }
}

impl SourceActions for GithubSource {
    /// Fetches the source to gather the required data to calculate a version bump. This source uses the GitHub REST API to
    /// gather the data.
    ///
    /// # Arguments
    ///
    /// * `sha` - git commit SHA from where the commits should be obtained. This SHA typically corresponds to the current
    /// HEAD commit.
    ///
    /// # Errors
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::ErrorKind::GithubRestError` if there was an unexpected
    /// response while using the GitHub REST API.
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::MissingGitTags` when not git tags were found.
    ///
    /// Returns `error::Error` with a kind of `error::ErrorKind::MissingGitOldestClosestTag` if the closest oldest closest tag
    /// could not be found.
    ///
    fn fetch_from_commit(&mut self, sha: &str) -> Result<(), Error> {
        let tags = get_all_tags(&self.repo_id, &self.token);
        if tags.is_empty() {
            return Err(Error::new(ErrorKind::MissingGitTags, None));
        }

        let commit_iterator = CommitIterator::new(&self.repo_id, &self.token, sha);
        for commit in commit_iterator {
            let commit = match commit {
                Ok(commit) => commit,
                Err(error) => return Err(error),
            };
            let tag = find_tag_from_commit_sha(&commit.sha, &tags);

            if let Some(tag) = tag {
                self.oldest_closest_tag = tag.clone().name;
                self.oldest_closest_tag_commit_sha = tag.commit.sha;
                break;
            }
            self.commit_messages.push(commit.commit.message);
        }

        if self.oldest_closest_tag.is_empty() {
            return Err(Error::new(ErrorKind::MissingGitOldestClosestTag, None));
        };

        self.remote_fetched = true;
        Ok(())
    }

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
            return Err(Error::new(
                ErrorKind::SourceNotFetched,
                Some("get_commit_messages"),
            ));
        }
        Ok(&self.commit_messages)
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
            return Err(Error::new(
                ErrorKind::SourceNotFetched,
                Some("get_closest_tag"),
            ));
        }
        Ok(&self.oldest_closest_tag)
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

/// Type used to iterate over GitHub commits. This type implements the `Iterator` trait
/// and performs paginated requests to the GitHub REST API.
struct CommitIterator<'a> {
    page: u64,
    per_page: u64,
    commits: Vec<GithubCommitDetails>,
    repo_id: &'a String,
    token: &'a Option<String>,
    sha: &'a str,

    max_elem: u64,
    current_elem: u64,
}

impl<'a> Iterator for CommitIterator<'a> {
    type Item = Result<GithubCommitDetails, Error>;

    /// Iterates over GitHub commits obtained by using paginated requests with 100 elements per page to the
    /// GitHub REST API.
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_elem == self.max_elem {
            self.commits = match get_commits_from_commit_sha(
                self.repo_id,
                self.sha,
                self.token,
                &self.page,
                &self.per_page,
            ) {
                Ok(commits) => commits,
                Err(error) => {
                    return Some(Err(error));
                }
            };

            // max_element max value will be github per_page max element, which is currently 100
            // and will """never""" exceed u64 size.
            self.max_elem = self.commits.len() as u64;
            self.current_elem = 0;
            self.page += 1;
        }

        let commit = self.commits.get(self.current_elem as usize);
        self.current_elem += 1;
        Ok(commit.cloned()).transpose()
    }
}

impl<'a> CommitIterator<'a> {
    /// Returns a new instance of a `CommitIterator`.
    fn new(repo_id: &'a String, token: &'a Option<std::string::String>, sha: &'a str) -> Self {
        CommitIterator {
            page: 0,
            per_page: DEFAULT_PER_PAGE,
            commits: vec![],
            repo_id,
            token,
            sha,
            max_elem: 0,
            current_elem: 0,
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
    token: &Option<String>,
    page: &u64,
    per_page: &u64,
) -> Result<Vec<GithubTag>, Error> {
    let client = reqwest::blocking::Client::new();
    let mut client = client
        .get(format!(
            "{}/{}{}?page={}&per_page={}",
            GITHUB_BASE_URI, repo_id, GITHUB_TAGS_URI, page, per_page
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

fn get_all_tags(repo_id: &String, token: &Option<String>) -> Vec<GithubTag> {
    let mut page: u64 = 0;
    let mut tags: Vec<GithubTag> = vec![];

    while let Ok(t) = get_tags(repo_id, token, &page, &DEFAULT_PER_PAGE) {
        if t.is_empty() {
            break;
        }

        tags.reserve(t.len());
        tags.extend(t);
        page += 1;
    }
    return tags;
}

/// Obtains commits from the given `sha` using the GitHub REST API. If `token` is given, the requests will be authorized.
/// Requests to GitHub REST API are paginated.
///
/// # Arguments
///
/// * `repo_id` - GitHub repository identifier that will be used to query commits.
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
    sha: &str,
    token: &Option<String>,
    page: &u64,
    per_page: &u64,
) -> Result<Vec<GithubCommitDetails>, Error> {
    let client = reqwest::blocking::Client::new();
    let mut client = client
        .get(format!(
            "{}/{}{}?sha={}&page={}&per_page={}",
            GITHUB_BASE_URI, repo_id, GITHUB_COMMITS_URI, sha, page, per_page
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
fn find_tag_from_commit_sha(sha: &str, tags: &Vec<GithubTag>) -> Option<GithubTag> {
    for tag in tags {
        if tag.commit.sha == sha {
            return Some((*tag).clone());
        }
    }
    None
}

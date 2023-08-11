use crate::error::{Error, ErrorKind};
use crate::source::SourceActions;
use reqwest;
use serde::Deserialize;

pub struct GithubSource {
    commit_messages: Vec<String>,
    closest_tag: String,
    closest_tag_commit_sha: String,
    remote_fetched: bool,

    repo_id: String,
    token: Option<String>,
}

impl GithubSource {
    pub fn new(repo_id: String, token: Option<String>) -> Self {
        Self {
            commit_messages: vec![],
            closest_tag: "".to_owned(),
            closest_tag_commit_sha: "".to_owned(),
            remote_fetched: false,
            repo_id,
            token,
        }
    }
}

impl SourceActions for GithubSource {
    fn fetch_from_commit(&mut self, sha: String) -> Result<(), Error> {
        let tags = match get_tags(&self.repo_id, &self.token)? {
            Some(tags) => tags,
            None => return Err(Error::new(ErrorKind::MissingGitTags, None)),
        };

        let commits = match get_commits_from_commit_sha(&self.repo_id, &sha, &self.token)? {
            Some(commits) => commits,
            None => return Err(Error::new(ErrorKind::MissingGitCommits, None)),
        };

        for commit in commits {
            let tag = find_tag_from_commit_sha(commit.sha, &tags);

            if let Some(tag) = tag {
                self.closest_tag = tag.clone().name;
                self.closest_tag_commit_sha = tag.commit.sha;
                break;
            }
            self.commit_messages.push(commit.commit.message);
        }

        if self.closest_tag.is_empty() {
            return Err(Error::new(ErrorKind::MissingGitClosestTag, None));
        };

        self.remote_fetched = true;
        Ok(())
    }

    fn get_commit_messages(&self) -> Result<&Vec<String>, Error> {
        if !self.remote_fetched {
            return Err(Error::new(
                ErrorKind::SourceNotFetched,
                Some("get_commit_messages"),
            ));
        }
        Ok(&self.commit_messages)
    }

    fn get_closest_tag(&self) -> Result<&String, Error> {
        if !self.remote_fetched {
            return Err(Error::new(
                ErrorKind::SourceNotFetched,
                Some("get_closest_tag"),
            ));
        }
        Ok(&self.closest_tag)
    }
}

const GITHUB_BASE_URI: &str = "https://api.github.com/repos";
const GITHUB_TAGS_URI: &str = "/tags";
const GITHUB_COMMITS_URI: &str = "/commits";
const USER_AGENT: &str = "tag-track";
const AUTH_HEADER: &str = "authorization";

#[derive(Debug, Deserialize, Clone)]
struct GithubTag {
    name: String,
    commit: GithubTagCommit,
}

#[derive(Debug, Deserialize, Clone)]
struct GithubTagCommit {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GithubCommitDetails {
    sha: String,
    commit: GithubCommit,
}

#[derive(Debug, Deserialize)]
struct GithubCommit {
    message: String,
}

fn get_tags(repo_id: &String, token: &Option<String>) -> Result<Option<Vec<GithubTag>>, Error> {
    let client = reqwest::blocking::Client::new();
    let mut client = client
        .get(format!(
            "{}/{}{}",
            GITHUB_BASE_URI, repo_id, GITHUB_TAGS_URI
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

    match tags.len() {
        0 => Ok(None),
        _ => Ok(Some(tags)),
    }
}

fn get_commits_from_commit_sha(
    repo_id: &String,
    sha: &String,
    token: &Option<String>,
) -> Result<Option<Vec<GithubCommitDetails>>, Error> {
    let client = reqwest::blocking::Client::new();
    let mut client = client
        .get(format!(
            "{}/{}{}?sha={}",
            GITHUB_BASE_URI, repo_id, GITHUB_COMMITS_URI, sha
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

    match commits.len() {
        0 => Ok(None),
        _ => Ok(Some(commits)),
    }
}

fn find_tag_from_commit_sha(sha: String, tags: &Vec<GithubTag>) -> Option<GithubTag> {
    for tag in tags {
        if tag.commit.sha == sha {
            return Some((*tag).clone());
        }
    }
    None
}

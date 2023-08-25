use crate::error::Error;
use enum_dispatch::enum_dispatch;

pub mod git;
pub mod github;

#[enum_dispatch]
pub trait SourceActions {
    fn get_commit_messages(&self) -> Result<&Vec<String>, Error>;
    fn get_closest_tag(&self) -> Result<&String, Error>;
    fn fetch_from_commit(&mut self, sha: &str) -> Result<(), Error>;
}

#[enum_dispatch(SourceActions)]
pub enum SourceKind {
    Git(git::GitSource),
    Github(github::GithubSource),
}

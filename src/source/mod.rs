use enum_dispatch::enum_dispatch;
use crate::error::ErrorKind;

pub mod git;

#[enum_dispatch]
pub trait SourceActions {
    fn get_commits(&self) -> Result<&Vec<String>, ErrorKind>;
    fn from_commit_sha(&mut self);
}

#[enum_dispatch(SourceActions)]
pub enum SourceKind {
    Git(git::GitSource)
}
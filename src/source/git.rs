use crate::error::{Error, ErrorKind};
use crate::git;
use crate::source::SourceActions;

pub struct GitSource {
    commit_messages: Vec<String>,
    closest_tag: String,
    closest_tag_commit_sha: String,
    remote_fetched: bool,
}

impl GitSource {
    pub fn new() -> Self {
        Self {
            commit_messages: vec![],
            closest_tag: "".to_owned(),
            closest_tag_commit_sha: "".to_owned(),
            remote_fetched: false,
        }
    }
}

impl SourceActions for GitSource {
    fn get_commit_messages(&self) -> Result<&Vec<String>, Error> {
        if !self.remote_fetched {
            return Err(Error::new(ErrorKind::SourceNotFetched, None));
        }
        Ok(&self.commit_messages)
    }

    fn fetch_from_commit(&mut self, sha: String) -> Result<(), Error> {
        self.closest_tag = match git::get_closest_tag() {
            Ok(tag) => tag,
            Err(error) => return Err(error),
        };

        self.closest_tag_commit_sha = match git::get_tag_commit_sha(&self.closest_tag) {
            Ok(tag_commit_sha) => tag_commit_sha,
            Err(error) => return Err(error),
        };

        self.commit_messages = match git::get_commit_messages(&self.closest_tag_commit_sha, &sha) {
            Ok(commit_messages) => commit_messages,
            Err(error) => return Err(error),
        };

        self.remote_fetched = true;
        Ok(())
    }

    fn get_closest_tag(&self) -> Result<&String, Error> {
        if !self.remote_fetched {
            return Err(Error::new(ErrorKind::SourceNotFetched, None));
        }
        Ok(&self.closest_tag)
    }
}

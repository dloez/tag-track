use crate::error::ErrorKind;
use crate::source::SourceActions;

pub struct GitSource {
    commits: Vec<String>,
    remote_fetched: bool,
}

impl GitSource {
    pub fn new() -> Self {
        Self {
            commits: vec![],
            remote_fetched: false,
        }
    }
}

impl SourceActions for GitSource {
    fn get_commits(&self) -> Result<&Vec<String>, ErrorKind> {
        if !self.remote_fetched {
            return Err(ErrorKind::SourceNotFetched);
        }
        Ok(&self.commits)
    }

    fn from_commit_sha(&mut self) {
        self.remote_fetched = true;
    }
}

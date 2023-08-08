#[derive(Debug)]
pub enum ErrorKind {
    GenericCommandFailed,
    MissingGit,
    NotGitWorkingTree,
    SourceNotFetched,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(error_kind: ErrorKind, message: String) -> Self {
        Self {
            kind: error_kind,
            message: message,
        }
    }
}

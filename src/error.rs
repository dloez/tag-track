#[derive(Debug)]
pub enum ErrorKind {
    GenericCommandFailed(String),
    MissingGit,
    NotGitWorkingTree,
    SourceNotFetched,
}

pub struct Error {
    payload: String
}

impl Error {
    fn new(error_kind: ErrorKind, payload: &str) -> Self {
        let repr = format!()
        Self {
            payload: payload.to_owned()
        }
    }
}

use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    GenericCommandFailed,
    MissingGit,
    NotGitWorkingTree,
    SourceNotFetched,
    GithubRestError,
    MissingGitTags,
    MissingGitCommits,
    MissingGitClosestTag,
    Other,
}

impl ErrorKind {
    pub fn as_str(&self) -> &str {
        use ErrorKind::*;

        match *self {
            GenericCommandFailed => "shell command failed",
            MissingGit => "missing git installation",
            NotGitWorkingTree => "the current directory does not seem to be a git working tree",
            SourceNotFetched => "call `fetch` method before using this property",
            GithubRestError => "error while calling GitHub REST API",
            MissingGitTags => "ther are no tags in source",
            MissingGitCommits => "there are no commits in source",
            MissingGitClosestTag => "cannot find closest tag",
            Other => "other error",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: Option<&str>) -> Self {
        let message = message.unwrap_or("");
        Self {
            kind,
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}: {}", self.kind.as_str(), self.message)
    }
}

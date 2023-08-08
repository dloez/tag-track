use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    GenericCommandFailed,
    MissingGit,
    NotGitWorkingTree,
    SourceNotFetched,
}

impl ErrorKind {
    pub fn as_str(&self) -> &str {
        use ErrorKind::*;

        match *self {
            GenericCommandFailed => "shell command failed",
            MissingGit => "missing git installation",
            NotGitWorkingTree => "the current directory does not seem to be a git working tree",
            SourceNotFetched => "call `fetch` before using this property"
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}

pub struct Error {
    pub kind: ErrorKind,
    message: String
}

impl Error {
    pub fn new(kind: ErrorKind, message: Option<&str>) -> Self {
        let message = message.unwrap_or("");
        Self {
            kind,
            message: message.to_owned()
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}: {}", self.kind.as_str(), self.message)
    }
}

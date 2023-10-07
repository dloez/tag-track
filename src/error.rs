//! Module containing error utilities. The `error::Error` struct does also implement the `From` trait
//! to convert other errors to allow the application to return our own errors in all functions.
//!

use std::{env::VarError, fmt};

/// Describes what kind of errors the application can return.
#[derive(Debug)]
pub enum ErrorKind {
    /// Unexpected error obtained while calling a command.
    GenericCommandFailed,
    /// Git command cannot be invoked.
    MissingGit,
    /// The application is being called outside a git working tree.
    NotGitWorkingTree,
    /// The source function `fetch_from_commit` or equivalent has not been called,
    /// which means that the source has not collected the required information and
    /// cannot be queried.
    SourceNotFetched,
    /// Error returned by the GitHub REST API.
    GithubRestError,
    /// Can not find any tags.
    MissingGitTags,
    /// Can not find the git oldest closest tag.
    MissingGitOldestClosestTag,
    /// The user given output format is not valid.
    InvalidOutputFormat,
    /// The regex pattern is not valid.
    InvalidRegexPattern,
    /// The commit pattern does not match the commit.
    InvalidCommitPattern,
    /// The tag pattern does not match the tag.
    InvalidTagPattern,
    /// The tag does not contain a version.
    NoVersionInTag,
    /// Unspecified found error. This error kind is also used for `From` implementation of
    /// other errors.
    Other,
}

impl ErrorKind {
    /// Create `&str` representation of the different error kinds.
    pub fn as_str(&self) -> &str {
        use ErrorKind::*;

        match *self {
            GenericCommandFailed => "shell command failed",
            MissingGit => "missing git installation",
            NotGitWorkingTree => "the current directory does not seem to be a git working tree",
            SourceNotFetched => "call `fetch` method before using this property",
            GithubRestError => "error while calling GitHub REST API",
            MissingGitTags => "there are no tags in source",
            MissingGitOldestClosestTag => "cannot find closest tag",
            InvalidOutputFormat => "the specified output format is not valid",
            InvalidRegexPattern => "the regex pattern is not valid",
            InvalidCommitPattern => "the commit pattern does not match the commit",
            InvalidTagPattern => "the tag pattern does not match the tag",
            NoVersionInTag => "the tag does not contain a version",
            Other => "other error",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_str().trim())
    }
}

/// Representation of an error.
#[derive(Debug)]
pub struct Error {
    /// Kind of encapsulated error.
    pub kind: ErrorKind,
    /// Error message or description for a better understanding. This `String` can be
    /// empty in case the error does not required a message or description.
    message: String,
}

impl Error {
    /// Create a new Error instance with the given kind and message.
    ///
    /// # Arguments
    ///
    /// * `kind` - Encapsulated `error::ErrorKind`.
    ///
    /// * `message` - Error message or description.
    ///
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
        let output = match self.message.is_empty() {
            true => self.kind.as_str().to_string(),
            false => format!("{}: {}", self.kind.as_str(), self.message.trim()),
        };
        write!(fmt, "{}", output)
    }
}

impl From<VarError> for Error {
    fn from(error: VarError) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string(),
        }
    }
}

impl From<semver::Error> for Error {
    fn from(error: semver::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string(),
        }
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string(),
        }
    }
}

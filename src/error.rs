//! Module containing error utilities. The `error::Error` struct does also implement the `From` trait
//! to convert other errors to allow the application to return our own errors in all functions.
//!

use std::{env::VarError, fmt};

/// Describes what kind of errors the application can return.
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    /// Unexpected error obtained while calling a command.
    GenericCommandFailed,
    /// Git command cannot be invoked.
    MissingGit,
    /// The application is being called outside a git working tree.
    NotGitWorkingTree,
    /// Error returned by the GitHub REST API.
    GithubRestError,
    /// Can not get tags from source.
    MissingGitTags,
    /// The user given output format is not valid.
    InvalidOutputFormat,
    /// The regex pattern is not valid.
    InvalidRegexPattern,
    /// Authentication is required for the action you are trying to call.
    AuthenticationRequired,
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
            GithubRestError => "error while calling GitHub REST API",
            MissingGitTags => "cannot get tags from source",
            InvalidOutputFormat => "the specified output format is not valid",
            InvalidRegexPattern => "the regex pattern is not valid",
            AuthenticationRequired => {
                "authentication is required for the action you are trying to call"
            }
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
        let message = message.unwrap_or("").replace('\n', " # ");
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
            message: error.to_string().replace('\n', " # "),
        }
    }
}

impl From<semver::Error> for Error {
    fn from(error: semver::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string().replace('\n', " # "),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string().replace('\n', " # "),
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string().replace('\n', " # "),
        }
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Self {
            kind: ErrorKind::Other,
            message: error.to_string().replace('\n', " # "),
        }
    }
}

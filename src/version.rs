//! Module containing version related help functions and types to define version increments.
//!
//! Version increments follow the Semantic Versioning 2.0
//!

use crate::{
    config::BumpRule,
    error::{Error, ErrorKind},
    git::Commit,
};
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};
use serde::Deserialize;

/// Types for different version.
/// The increment types follow the Semantic Version specification.
#[derive(Eq, PartialEq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncrementKind {
    /// Increment the major section of a version.
    Major,
    /// Increment the minor section of a version.
    Minor,
    /// Increment the patch section of a version.
    Patch,
}

/// Increments the patch section of a `semver::Version`. This also empties the pre-release
/// and build sections.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
pub fn increment_patch(version: &mut Version) {
    version.patch += 1;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
}

/// Increments the minor section of a `semver::Version`. This also empties the pre-release
/// and build sections, and sets the patch section to 0.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
pub fn increment_minor(version: &mut Version) {
    version.minor += 1;
    version.patch = 0;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
}

/// Increments the major section of a `semver::Version`. This also empties the pre-release
/// and build sections, and sets the patch and minor section to 0.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
pub fn increment_major(version: &mut Version) {
    version.major += 1;
    version.minor = 0;
    version.patch = 0;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
}

/// Increments the given version based on the given rules. The function returns the increment kind that was used.
/// If no increment was done, the function returns `None`.
///
/// # Arguments
///
/// * `version` - Version that will be modified.
///
/// * `rules` - Rules that will be used to determine the increment kind.
///
/// * `commits` - Commits that will be used to determine the increment kind.
///
/// * `commit_pattern` - Pattern that will be used to parse the conventional commit.
///
/// # Errors
///
/// Returns `error::Error` with the type of `error::ErrorKind::InvalidCommitPattern` if the given Regex pattern for the commit is not a valid.
///
pub fn bump_version(
    version: &mut Version,
    rules: &Vec<BumpRule>,
    commits: &Vec<Commit>,
    commit_pattern: &str,
) -> Result<Option<IncrementKind>, Error> {
    let mut increment_kind: Option<IncrementKind> = None;
    'commits: for commit in commits {
        let commit_details = match extract_commit_details(&commit, commit_pattern) {
            Ok(commit_details) => commit_details,
            Err(error) => match error.kind {
                ErrorKind::InvalidCommitPattern => {
                    println!("commit {} does not match the commit pattern", commit.sha);
                    continue;
                }
                _ => return Err(error),
            },
        };

        for rule in rules {
            let mut bump = false;

            // Check commit type
            if let Some(types) = &rule.types {
                if types.contains(&commit_details.commit_type) {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check commit scope
            if let Some(scopes) = &rule.scopes {
                if scopes.contains(&commit_details.scope) {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check breaking field
            if let Some(if_breaking_field) = &rule.if_breaking_field {
                if *if_breaking_field && commit_details.breaking {
                    bump = true;
                } else {
                    continue;
                }
            }

            // Check breaking description
            if let Some(if_breaking_description) = &rule.if_breaking_description {
                if *if_breaking_description
                    && (commit_details.description.contains("BREAKING CHANGE")
                        || commit_details.description.contains("BREAKING-CHANGE"))
                {
                    bump = true;
                } else {
                    continue;
                }
            }

            if bump {
                increment_kind = match &rule.bump {
                    IncrementKind::Major => Some(IncrementKind::Major),
                    IncrementKind::Minor => Some(IncrementKind::Minor),
                    IncrementKind::Patch => {
                        if increment_kind.is_some() {
                            continue;
                        }
                        Some(IncrementKind::Minor)
                    }
                };

                if let Some(IncrementKind::Major) = increment_kind {
                    break 'commits;
                }
            }
        }
    }

    match increment_kind {
        Some(IncrementKind::Major) => increment_major(version),
        Some(IncrementKind::Minor) => increment_minor(version),
        Some(IncrementKind::Patch) => increment_patch(version),
        None => {}
    };

    Ok(increment_kind)
}

/// Type to represent the sections of a conventional commit message.
#[derive(Debug)]
struct CommitDetails {
    /// The type of the commit.
    commit_type: String,

    /// The scope of the commit.
    scope: String,

    /// If the commit includes a breaking change. Typically this is true if the commit type includes the `!` char.
    breaking: bool,

    /// The description of the conventional commit.
    description: String,
}

const TYPE_FIELD: &str = "type";
const SCOPE_FIELD: &str = "scope";
const BREAKING_FIELD: &str = "breaking";
const DESCRIPTION_FIELD: &str = "description";

/// Extracts the commit details from a commit message.
///
/// # Arguments
///
/// * `commit` - Commit message that will be parsed.
///
/// * `commit_pattern` - Pattern that will be used to parse the conventional commit.
///
/// # Errors
///
/// Returns `error::Error` with the type of `error::ErrorKind::InvalidCommitPattern` if the given Regex pattern for the commit is not a valid.
///
fn extract_commit_details(commit: &Commit, commit_pattern: &str) -> Result<CommitDetails, Error> {
    let re = match Regex::new(commit_pattern) {
        Ok(re) => re,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidRegexPattern,
                Some(format!("{} - {}", commit_pattern, error.to_string().as_str()).as_str()),
            ))
        }
    };

    let captures = match re.captures(&commit.message) {
        Some(captures) => captures,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some(
                    format!(
                        "commit {} does not match pattern {}",
                        commit.message, commit_pattern
                    )
                    .as_str(),
                ),
            ))
        }
    };

    let commit_type = match captures.name(TYPE_FIELD) {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some("missing commit type"),
            ))
        }
        Some(found_match) => found_match.as_str().trim().to_string(),
    };

    let scope = match captures.name(SCOPE_FIELD) {
        None => "".to_string(),
        Some(found_match) => found_match
            .as_str()
            .replace("(", "")
            .replace(")", "")
            .trim()
            .to_string(),
    };

    let breaking = match captures.name(BREAKING_FIELD) {
        None => false,
        Some(_) => true,
    };

    let description = match captures.name(DESCRIPTION_FIELD) {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidCommitPattern,
                Some("missing commit description"),
            ))
        }
        Some(found_match) => {
            if found_match.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidCommitPattern,
                    Some("missing commit description"),
                ));
            }
            found_match.as_str().trim().to_string()
        }
    };

    Ok(CommitDetails {
        commit_type,
        scope,
        breaking,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_can_be_extracted() {
        let commit = Commit {
            sha: "1234567890".to_string(),
            message: "feat(scope): add new feature".to_string(),
        };
        let commit_details =
            extract_commit_details(&commit, crate::config::DEFAULT_COMMIT_PATTERN).unwrap();
        assert_eq!(commit_details.commit_type, "feat");
        assert_eq!(commit_details.scope, "scope");
        assert_eq!(commit_details.breaking, false);
        assert_eq!(commit_details.description, "add new feature");
        let commit = Commit {
            sha: "1234567890".to_string(),
            message: "feat(scope)!: add new feature".to_string(),
        };

        let commit_details =
            extract_commit_details(&commit, crate::config::DEFAULT_COMMIT_PATTERN).unwrap();
        assert_eq!(commit_details.commit_type, "feat");
        assert_eq!(commit_details.scope, "scope");
        assert_eq!(commit_details.breaking, true);
        assert_eq!(commit_details.description, "add new feature");

        let commit = Commit {
            sha: "1234567890".to_string(),
            message: "feat: add new feature".to_string(),
        };
        let commit_details =
            extract_commit_details(&commit, crate::config::DEFAULT_COMMIT_PATTERN).unwrap();
        assert_eq!(commit_details.commit_type, "feat");
        assert_eq!(commit_details.scope, "");
        assert_eq!(commit_details.breaking, false);
        assert_eq!(commit_details.description, "add new feature");

        let commit = Commit {
            sha: "1234567890".to_string(),
            message: "feat: add new feature\n\nBREAKING CHANGE: this is a breaking change"
                .to_string(),
        };
        let commit_details =
            extract_commit_details(&commit, crate::config::DEFAULT_COMMIT_PATTERN).unwrap();
        assert_eq!(commit_details.commit_type, "feat");
        assert_eq!(commit_details.scope, "");
        assert_eq!(commit_details.breaking, false);
        assert_eq!(
            commit_details.description,
            "add new feature\n\nBREAKING CHANGE: this is a breaking change"
        );

        let commit = Commit {
            sha: "1234567890".to_string(),
            message: "feat:".to_string(),
        };
        let commit_details = extract_commit_details(&commit, crate::config::DEFAULT_COMMIT_PATTERN);
        println!("{:?}", commit_details);
        assert!(commit_details.is_err());

        let commit = Commit {
            sha: "1234567890".to_string(),
            message: "example".to_string(),
        };
        let commit_details = extract_commit_details(&commit, crate::config::DEFAULT_COMMIT_PATTERN);
        assert!(commit_details.is_err());
    }

    #[test]
    fn bump_version_major() {
        let mut version = Version::parse("1.2.3").unwrap();
        let mut commits = Vec::new();
        commits.push(Commit {
            sha: "1234567890".to_string(),
            message: "feat(scope): add new feature".to_string(),
        });
        let mut rules = Vec::new();
        rules.push(BumpRule {
            types: Some(vec!["feat".to_string()]),
            scopes: None,
            if_breaking_field: None,
            if_breaking_description: None,
            bump: IncrementKind::Major,
        });
        let increment_kind = bump_version(
            &mut version,
            &rules,
            &commits,
            crate::config::DEFAULT_COMMIT_PATTERN,
        )
        .unwrap();
        assert_eq!(increment_kind, Some(IncrementKind::Major));
        assert_eq!(version.to_string(), "2.0.0");

        let mut version = Version::parse("1.2.3").unwrap();
        let mut commits = Vec::new();
        commits.push(Commit {
            sha: "1234567890".to_string(),
            message: "feat(scope): add new feature".to_string(),
        });
        let mut rules = Vec::new();
        rules.push(BumpRule {
            types: None,
            scopes: Some(vec!["scope".to_string()]),
            if_breaking_field: None,
            if_breaking_description: None,
            bump: IncrementKind::Major,
        });
        let increment_kind = bump_version(
            &mut version,
            &rules,
            &commits,
            crate::config::DEFAULT_COMMIT_PATTERN,
        )
        .unwrap();
        assert_eq!(increment_kind, Some(IncrementKind::Major));
        assert_eq!(version.to_string(), "2.0.0");

        let mut version = Version::parse("1.2.3").unwrap();
        let mut commits = Vec::new();
        commits.push(Commit {
            sha: "1234567890".to_string(),
            message: "feat(scope)!: add new feature".to_string(),
        });
        let mut rules = Vec::new();
        rules.push(BumpRule {
            types: None,
            scopes: None,
            if_breaking_field: Some(true),
            if_breaking_description: None,
            bump: IncrementKind::Major,
        });
        let increment_kind = bump_version(
            &mut version,
            &rules,
            &commits,
            crate::config::DEFAULT_COMMIT_PATTERN,
        )
        .unwrap();
        assert_eq!(increment_kind, Some(IncrementKind::Major));
        assert_eq!(version.to_string(), "2.0.0");
    }

    #[test]
    fn increment_patch_section() {
        let mut version = Version::parse("1.2.3").unwrap();
        increment_patch(&mut version);
        assert_eq!(version.to_string(), "1.2.4");
    }

    #[test]
    fn increment_minor_section() {
        let mut version = Version::parse("1.2.3").unwrap();
        increment_minor(&mut version);
        assert_eq!(version.to_string(), "1.3.0");
    }

    #[test]
    fn increment_major_section() {
        let mut version = Version::parse("1.2.3").unwrap();
        increment_major(&mut version);
        assert_eq!(version.to_string(), "2.0.0");
    }
}

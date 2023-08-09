use std::{process::exit, collections::HashMap};

use regex::Regex;
use source::SourceActions;

mod error;
mod git;
mod source;
mod version;

const MAJOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf)!:";
const MINOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf):";
const PATCH_REGEX_PATTERN: &str = r"^fix:";

fn main() {
    if let Err(error) = git::verify_git() {
        println!("{}", error);
        exit(1);
    }

    let current_commit = match git::get_current_commit() {
        Ok(current_commit) => current_commit,
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };

    let mut git_source = source::git::GitSource::new();
    if let Err(error) = git_source.fetch_from_commit(current_commit) {
        println!("{}", error);
        exit(1);
    };

    let git_source = git_source;
    let version = 
    let commit_messages = match git_source.get_commit_messages() {
        Ok(commit_messages) => commit_messages,
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };

    let patterns: HashMap<version::IncrementKind, &'static str> = HashMap::from([
        (version::IncrementKind::Major, MAJOR_REGEX_PATTERN),
        (version::IncrementKind::Patch, PATCH_REGEX_PATTERN),
        (version::IncrementKind::Minor, MINOR_REGEX_PATTERN),
    ]);

    let mut increment_kind: Option<&version::IncrementKind> = None;
    for commit_message in commit_messages {
        for (kind, pattern) in &patterns {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(commit_message.as_str()) {
                match kind {
                    version::IncrementKind::Major => {
                        increment_kind = Some(kind);
                        break;
                    }
                    version::IncrementKind::Patch => increment_kind = Some(kind),
                    version::IncrementKind::Minor => {
                        if increment_kind.is_some() {
                            continue;
                        }
                        increment_kind = Some(&version::IncrementKind::Minor)
                    }
                }
            }
        }
    }
    let increment_kind = increment_kind;

    if increment_kind.is_none() {
        println!("version bump not required");
        exit(0);
    }

    let kind = increment_kind.unwrap();
    print!("version change: {} -> ", version);
    match kind {
        IncrementKind::Major => increment_major(&mut version),
        IncrementKind::Minor => increment_minor(&mut version),
        IncrementKind::Patch => increment_patch(&mut version),
    }
    println!("{}", version);
}

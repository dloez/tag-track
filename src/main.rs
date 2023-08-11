use clap::Parser;
use regex::Regex;
use semver::Version;
use source::SourceActions;
use std::env;
use std::{collections::HashMap, process::exit};

mod error;
mod git;
mod source;
mod version;

const MAJOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf)!:";
const MINOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf):";
const PATCH_REGEX_PATTERN: &str = r"^fix:";

const GITHUB_ACTION_COMMIT_SHA_VAR: &str = "GITHUB_SHA";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Create git annotated tag from populated version.
    #[arg(short, long, default_value = "false", default_missing_value = "true")]
    create_tag: bool,

    // GitHUb repository identifier (owner/repo_name).
    // If pressent, this will use GitHub as the source to calculate a version bump.
    #[arg(short, long, default_value = None)]
    github_repo: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Err(error) = git::verify_git() {
        println!("{}", error);
        exit(1);
    }

    let current_commit_sha = match args.github_repo {
        Some(_) => match env::var(GITHUB_ACTION_COMMIT_SHA_VAR) {
            Ok(commit_sha) => commit_sha,
            Err(error) => {
                println!("{}", error);
                exit(1);
            }
        },
        None => match git::get_current_commit_sha() {
            Ok(current_commit) => current_commit,
            Err(error) => {
                println!("{}", error);
                exit(1);
            }
        },
    };

    let mut source: source::SourceKind = match args.github_repo {
        Some(repo) => source::SourceKind::Github(source::github::GithubSource::new(repo)),
        None => source::SourceKind::Git(source::git::GitSource::new()),
    };

    if let Err(error) = source.fetch_from_commit(current_commit_sha) {
        println!("{}", error);
        exit(1);
    };

    let source = source;
    let closest_tag = match source.get_closest_tag() {
        Ok(tag) => tag,
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };

    let mut version = match Version::parse(closest_tag) {
        Ok(version) => version,
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };

    let commit_messages = match source.get_commit_messages() {
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
        version::IncrementKind::Major => version::increment_major(&mut version),
        version::IncrementKind::Minor => version::increment_minor(&mut version),
        version::IncrementKind::Patch => version::increment_patch(&mut version),
    }
    println!("{}", version);

    if !args.create_tag {
        exit(0);
    }

    let tag_message = format!("Version {}", version);
    let result = git::create_tag(&version.to_string(), &tag_message);
    match result {
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
        Ok(_) => println!("tag '{}' created!", version),
    }
}

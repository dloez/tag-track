use clap::Parser;
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};
use std::{
    collections::HashMap,
    io::Error,
    io::ErrorKind,
    process::{exit, Command},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run with verbose mode
    #[arg(short, long, default_value = "false", default_missing_value = "true")]
    verbose: bool,
}

const MAJOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf)!:";
const MINOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf):";
const PATCH_REGEX_PATTERN: &str = r"^fix:";

#[derive(Eq, PartialEq, Hash)]
enum IncrementKind {
    Major,
    Minor,
    Patch,
}

fn verify_git() -> Result<bool, Error> {
    Command::new("git").arg("--version").output()?;

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()?;

    if !output.status.success() {
        return Err(Error::new(ErrorKind::Other, "not in a git working tree"));
    }
    Ok(true)
}

fn get_closest_tag() -> Result<String, Error> {
    let output = Command::new("git")
        .arg("describe")
        .arg("--abbrev=0")
        .arg("--tags")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "can not get closest tag: {} - code: {}",
                stderr,
                output.status.code().unwrap()
            ),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

fn get_tag_commit_sha(tag: &String) -> Result<String, Error> {
    let output = Command::new("git")
        .arg("rev-list")
        .args(["-n", "1"])
        .arg(tag)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "can not get tag commit sha: {} - code: {}",
                stderr,
                output.status.code().unwrap()
            ),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

fn get_commit_messages(from_commit: &String, to_commit: &String) -> Result<Vec<String>, Error> {
    let output = Command::new("git")
        .arg("log")
        .arg("--format=%s")
        .arg(from_commit)
        .arg(to_commit)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "can not get commit between '{}' and '{}', stderr: {} - code: {}",
                from_commit,
                to_commit,
                stderr,
                output.status.code().unwrap()
            ),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout.lines().map(|s| s.to_owned()).collect())
}

fn increment_patch(v: &mut Version) {
    v.patch += 1;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_minor(v: &mut Version) {
    v.minor += 1;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_major(v: &mut Version) {
    v.major += 1;
    v.minor = 0;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn main() {
    let args = Args::parse();

    match verify_git() {
        Err(error) => {
            println!("git verification not passed, error: {}", error);
            exit(1);
        }
        Ok(_) => {
            if args.verbose {
                println!("found git installation and git managed project")
            };
        }
    }

    let closest_tag = match get_closest_tag() {
        Err(error) => {
            println!("could not get a tag, error: {}", error);
            exit(1);
        }
        Ok(tag) => {
            if args.verbose {
                println!("closest tag found: {}", tag);
            }
            tag
        }
    };

    let mut version = match Version::parse(&closest_tag) {
        Err(error) => {
            println!(
                "could not parse tag '{}' as a semantic version, error: {}",
                closest_tag, error
            );
            exit(1);
        }
        Ok(version) => {
            println!("current version: {}", version);
            version
        }
    };

    let tag_commit_sha = match get_tag_commit_sha(&closest_tag) {
        Err(error) => {
            println!(
                "could not get commit sha for tag '{}', error: {}",
                closest_tag, error
            );
            exit(1);
        }
        Ok(commit_sha) => {
            if args.verbose {
                println!(
                    "commit sha of commit pointed by tag '{}': {}",
                    closest_tag, commit_sha
                )
            }
            commit_sha
        }
    };

    let head_commit = String::from("HEAD");
    let commit_messages = match get_commit_messages(&tag_commit_sha, &head_commit) {
        Err(error) => {
            println!(
                "failed to get commit messages from commit '{}' to commit '{}', error: {}",
                tag_commit_sha, head_commit, error
            );
            exit(1);
        }
        Ok(commit_messages) => {
            if args.verbose {
                println!(
                    "extracted commit messages from commit '{}' to commit '{}'",
                    tag_commit_sha, head_commit
                );
                println!("commit messages:");
                for commit_message in &commit_messages {
                    println!("- {}", commit_message);
                }
            }
            commit_messages
        }
    };

    let patterns: HashMap<IncrementKind, &'static str> = HashMap::from([
        (IncrementKind::Major, MAJOR_REGEX_PATTERN),
        (IncrementKind::Patch, PATCH_REGEX_PATTERN),
        (IncrementKind::Minor, MINOR_REGEX_PATTERN),
    ]);

    let mut increment_kind: Option<&IncrementKind> = None;
    for commit_message in commit_messages {
        for (kind, pattern) in &patterns {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(commit_message.as_str()) {
                match kind {
                    IncrementKind::Major => {
                        increment_kind = Some(kind);
                        break;
                    }
                    IncrementKind::Patch => increment_kind = Some(kind),
                    IncrementKind::Minor => {
                        if increment_kind.is_some() {
                            continue;
                        }
                        increment_kind = Some(&IncrementKind::Minor)
                    }
                }
            }
        }
    }
    let increment_kind = increment_kind;

    if let Some(kind) = increment_kind {
        print!("version change: {} -> ", version);
        match kind {
            IncrementKind::Major => increment_major(&mut version),
            IncrementKind::Minor => increment_minor(&mut version),
            IncrementKind::Patch => increment_patch(&mut version),
        }
        println!("{}", version);
    } else {
        println!("version bump not required");
    }
}

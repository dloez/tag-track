use clap::Parser;
use config::{is_config_available, parse_config_file, Config};
use error::{Error, ErrorKind};
use regex::Regex;
use semver::Version;
use serde::Serialize;
use serde_json::to_string_pretty;
use source::SourceActions;
use std::process::exit;
use version::bump_version;

mod config;
mod error;
mod git;
mod source;
mod version;

/// Type that defines CLI arguments.
#[derive(Parser, Debug, Serialize, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Create git annotated tag from populated version.
    #[arg(long, default_value = "false", default_missing_value = "true")]
    create_tag: bool,

    /// GitHUb repository identifier (owner/repo_name).
    /// If present, this will use GitHub as the source to calculate a version bump.
    #[arg(long)]
    github_repo: Option<String>,

    /// Token to authenticate  GitHub REST API calls.
    #[arg(long)]
    github_token: Option<String>,

    /// All commits between the oldest closest tag and the one specified
    /// by this SHA will be used to calculate the version bump. Useful when using
    /// a remote repository with different git history as the local repository.
    #[arg(long)]
    commit_sha: Option<String>,

    /// Output format, possible values are: 'text', 'json'. Default value is 'text'.
    #[arg(long, default_value = "text", default_missing_value = "text")]
    output_format: String,
}

/// Type for storing the required data that needs to be printed in the terminal in different formats.
#[derive(Serialize, Debug)]
struct Output<'a> {
    inputs: &'a Args,
    created_tag: bool,
    old_version: String,
    new_version: String,
    error: String,
}

impl<'a> Output<'a> {
    /// Creates a new `Output` instance with the given `inputs`.
    fn new(inputs: &'a Args) -> Self {
        Self {
            inputs,
            created_tag: false,
            old_version: "".to_owned(),
            new_version: "".to_owned(),
            error: "".to_owned(),
        }
    }
}

/// Type for valid output formats.
enum OutputFormat {
    Text,
    Json,
}

fn main() {
    let args = Args::parse();

    let output_format = match args.output_format.as_str() {
        "text" => OutputFormat::Text,
        "json" => OutputFormat::Json,
        value => {
            let error = Error::new(ErrorKind::InvalidOutputFormat, Some(value));
            println!("{}", error);
            exit(1);
        }
    };

    let config = match is_config_available() {
        Some(config_file_path) => match parse_config_file(config_file_path) {
            Ok(config) => config,
            Err(error) => {
                print_error(error, &args, &output_format);
                exit(1);
            }
        },
        None => Config::new(),
    };

    if let Err(error) = git::verify_git() {
        print_error(error, &args, &output_format);
        exit(1);
    }

    let current_commit_sha: String = match &args.commit_sha {
        Some(commit_sha) => commit_sha.to_owned(),
        None => match git::get_current_commit_sha() {
            Ok(current_commit) => current_commit,
            Err(error) => {
                print_error(error, &args, &output_format);
                exit(1);
            }
        },
    };

    // TODO: This will not work when we have more sources
    let mut source: source::SourceKind = match args.github_repo.clone() {
        Some(repo) => source::SourceKind::Github(source::github::GithubSource::new(
            repo,
            args.github_token.clone(),
        )),
        None => source::SourceKind::Git(source::git::GitSource::new()),
    };

    if let Err(error) = source.fetch_from_commit(&current_commit_sha) {
        print_error(error, &args, &output_format);
        exit(1);
    };

    let source = source;
    let closest_tag = match source.get_closest_oldest_tag() {
        Ok(tag) => tag,
        Err(error) => {
            print_error(error, &args, &output_format);
            exit(1);
        }
    };
    let mut version = match parse_tag(config.tag_pattern, closest_tag) {
        Ok(version) => version,
        Err(error) => {
            print_error(error, &args, &output_format);
            exit(1);
        }
    };

    let commits = match source.get_commits() {
        Ok(commits) => commits,
        Err(error) => {
            print_error(error, &args, &output_format);
            exit(1);
        }
    };

    let mut output = Output::new(&args);
    output.old_version = version.to_string();
    let increment_kind = match bump_version(
        &mut version,
        &config.bump_rules,
        commits,
        &config.commit_pattern,
    ) {
        Ok(increment_kind) => increment_kind,
        Err(error) => {
            print_error(error, &args, &output_format);
            exit(1);
        }
    };
    output.new_version = version.to_string();

    if increment_kind.is_none() {
        match output_format {
            OutputFormat::Text => println!("version bump not required"),
            OutputFormat::Json => {
                let mut output = Output::new(&args);
                output.old_version = version.to_string();
                output.new_version = version.to_string();
                if let Ok(json_str) = to_string_pretty(&output) {
                    println!("{}", json_str);
                } else {
                    println!("could not serialize {:?}", output);
                }
            }
        }
        exit(0);
    }

    if let OutputFormat::Text = output_format {
        println!(
            "version change: {} -> {}",
            output.old_version, output.new_version
        )
    }

    if !args.create_tag {
        if let OutputFormat::Json = output_format {
            if let Ok(json_str) = to_string_pretty(&output) {
                println!("{}", json_str);
            } else {
                println!("could not serialize {:?}", output);
            }
        }
        exit(0);
    }

    let tag_message = format!("Version {}", version);
    let result = git::create_tag(&version.to_string(), &tag_message);
    match result {
        Err(error) => {
            print_error(error, &args, &output_format);
            exit(1);
        }
        Ok(_) => {
            output.created_tag = true;
            match output_format {
                OutputFormat::Text => println!("tag '{}' created!", version),
                OutputFormat::Json => {
                    if let Ok(json_str) = to_string_pretty(&output) {
                        println!("{}", json_str);
                    } else {
                        println!("could not serialize {:?}", output);
                    }
                }
            }
        }
    }
}

/// Print the given error in the given output format.
///
/// # Arguments
///
/// * `error` - Error to be displayed.
///
/// * `inputs` - User inputted cli arguments.
///
/// * `output_format` - Output format that will be used for printing the result. The output
/// will be prettified before being printed.
///
fn print_error(error: error::Error, inputs: &Args, output_format: &OutputFormat) {
    match output_format {
        OutputFormat::Text => println!("{}", error),
        OutputFormat::Json => {
            let mut output = Output::new(inputs);
            output.error = format!("{}", error);
            if let Ok(json_str) = to_string_pretty(&output) {
                println!("{}", json_str);
            } else {
                println!("could not serialize {:?}", output);
            }
        }
    }
}

/// Regex field name for the version inside a tag.
const VERSION_FIELD: &str = "version";

/// Returns the version found in the given tag that matches the given pattern.
///
/// # Arguments
///
/// * `tag_pattern` - Regex pattern that will be used against the tag.
///
/// * `tag` - Tag that will be parsed.
///
/// # Errors
///
/// Returns `error::Error` with the type of `error::ErrorKind::InvalidTagPattern` if the given Regex pattern is not a valid.
///
/// Returns `error::Error` with the type of `error::ErrorKind::NoVersionInTag` if the given tag does not contain a version.
///
fn parse_tag(tag_pattern: String, tag: &String) -> Result<Version, Error> {
    let re = match Regex::new(tag_pattern.as_str()) {
        Ok(re) => re,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidTagPattern,
                Some(error.to_string().as_str()),
            ))
        }
    };
    let captures = match re.captures(tag) {
        Some(captures) => captures,
        None => {
            return Err(Error::new(
                ErrorKind::NoVersionInTag,
                Some(
                    format!(
                        "no version found in tag {} with pattern {} asd",
                        tag, tag_pattern
                    )
                    .as_str(),
                ),
            ))
        }
    };
    let matched_version = match captures.name(VERSION_FIELD) {
        Some(version) => version.as_str().to_owned(),
        None => {
            return Err(Error::new(
                ErrorKind::NoVersionInTag,
                Some(
                    format!(
                        "no version found in tag {} with pattern {}",
                        tag, tag_pattern
                    )
                    .as_str(),
                ),
            ))
        }
    };

    match Version::parse(matched_version.as_str()) {
        Ok(version) => Ok(version),
        Err(error) => Err(Error::from(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_can_be_parsed() {
        let tag_pattern = r"v(?P<version>.*)";
        let tag = "v1.2.3".to_owned();
        let version = parse_tag(tag_pattern.to_owned(), &tag).unwrap();
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn parse_tag_invalid_pattern() {
        let tag_pattern = r"v(?P<version>\d+\.\d+\.\d+";
        let tag = "v1.2.3".to_owned();
        let error = parse_tag(tag_pattern.to_owned(), &tag).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InvalidTagPattern);
    }

    #[test]
    fn parse_tag_no_version() {
        let tag_pattern = r"v(?P<version>\d+\.\d+\.\d+)";
        let tag = "v1.2".to_owned();
        let error = parse_tag(tag_pattern.to_owned(), &tag).unwrap_err();
        assert_eq!(error.kind, ErrorKind::NoVersionInTag);
    }
}

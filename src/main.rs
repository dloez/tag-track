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

    /// GitHub URL. Defaults to 'https://api.github.com'.
    #[arg(
        long,
        default_value = source::github::GITHUB_API_BASE_URL,
        default_missing_value = source::github::GITHUB_API_BASE_URL
    )]
    github_api_url: String,

    /// GitHub repository identifier (owner/repo_name).
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
    /// User inputted CLI arguments.
    inputs: &'a Args,
    /// Configuration that was used.
    config: Option<&'a Config>,
    /// If a new tag was created.
    tag_created: bool,
    /// New tag that was created.
    new_tag: String,
    /// Old version before bumping.
    old_version: String,
    /// New version after bumping.
    new_version: String,
    /// Commits that were skipped during the version bump due to pattern missmatch.
    skipped_commits: Vec<&'a String>,
    /// Error message if any.
    error: String,
}

impl<'a> Output<'a> {
    /// Creates a new `Output` instance with the given `inputs`.
    fn new(inputs: &'a Args, config: Option<&'a Config>) -> Self {
        Self {
            inputs,
            config,
            tag_created: false,
            new_tag: "".to_owned(),
            old_version: "".to_owned(),
            new_version: "".to_owned(),
            skipped_commits: vec![],
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
                print_error(error, &args, &output_format, None);
                exit(1);
            }
        },
        None => Config::new(),
    };

    if let Err(error) = git::verify_git() {
        print_error(error, &args, &output_format, Some(&config));
        exit(1);
    }

    let current_commit_sha: String = match &args.commit_sha {
        Some(commit_sha) => commit_sha.to_owned(),
        None => match git::get_current_commit_sha() {
            Ok(current_commit) => current_commit,
            Err(error) => {
                print_error(error, &args, &output_format, Some(&config));
                exit(1);
            }
        },
    };

    // TODO: This will not work when we have more sources
    let mut source: source::SourceKind = match args.github_repo.clone() {
        Some(repo) => source::SourceKind::Github(source::github::GithubSource::new(
            repo,
            validate_trailing_slash(&args.github_api_url),
            args.github_token.clone(),
        )),
        None => source::SourceKind::Git(source::git::GitSource::new()),
    };

    if let Err(error) = source.fetch_from_commit(&current_commit_sha) {
        print_error(error, &args, &output_format, Some(&config));
        exit(1);
    };

    let source = source;
    let closest_tag = match source.get_closest_oldest_tag() {
        Ok(tag) => tag,
        Err(error) => {
            print_error(error, &args, &output_format, Some(&config));
            exit(1);
        }
    };
    let (mut version, start_match) = match parse_tag(&config.tag_pattern, closest_tag) {
        Ok(version) => version,
        Err(error) => {
            print_error(error, &args, &output_format, Some(&config));
            exit(1);
        }
    };

    let commits = match source.get_commits() {
        Ok(commits) => commits,
        Err(error) => {
            print_error(error, &args, &output_format, Some(&config));
            exit(1);
        }
    };

    let mut output = Output::new(&args, Some(&config));
    output.old_version = version.to_string();
    let (increment_kind, skipped_commits_sha) = match bump_version(
        &mut version,
        &config.bump_rules,
        commits,
        &config.commit_pattern,
    ) {
        Ok(increment_kind) => increment_kind,
        Err(error) => {
            print_error(error, &args, &output_format, Some(&config));
            exit(1);
        }
    };
    output.new_version = version.to_string();
    output.skipped_commits = skipped_commits_sha;

    if let OutputFormat::Text = output_format {
        output
            .skipped_commits
            .iter()
            .for_each(|sha| println!("commit '{}' does not match the commit pattern", sha));
    }

    if increment_kind.is_none() {
        match output_format {
            OutputFormat::Text => println!("version bump not required"),
            OutputFormat::Json => {
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
    let mut new_tag_name = closest_tag.clone();
    new_tag_name.replace_range(start_match.., version.to_string().as_str());
    match git::create_tag(&new_tag_name, &tag_message) {
        Err(error) => {
            print_error(error, &args, &output_format, Some(&config));
            exit(1);
        }
        Ok(_) => {
            output.tag_created = true;
            output.new_tag = new_tag_name.clone();
            match output_format {
                OutputFormat::Text => println!("tag '{}' created!", new_tag_name),
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
fn print_error(
    error: error::Error,
    inputs: &Args,
    output_format: &OutputFormat,
    config: Option<&Config>,
) {
    match output_format {
        OutputFormat::Text => println!("{}", error),
        OutputFormat::Json => {
            let mut output = Output::new(inputs, config);
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
fn parse_tag(tag_pattern: &String, tag: &String) -> Result<(Version, usize), Error> {
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
                        "no version found in tag {} with pattern {}",
                        tag, tag_pattern
                    )
                    .as_str(),
                ),
            ))
        }
    };

    let matched_version = match captures.name(VERSION_FIELD) {
        Some(version) => (version.as_str().to_owned(), version.start()),
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

    let parsed_version = Version::parse(matched_version.0.as_str())?;
    Ok((parsed_version, matched_version.1))
}

/// Validates the given URL and returns a valid URL without a trailing slash.
///
/// # Arguments
///
/// * `url` - URL to be validated.
///
fn validate_trailing_slash(url: &str) -> String {
    let mut url = url.to_owned();
    if url.ends_with('/') {
        url.pop();
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_can_be_parsed() {
        let tag_pattern = r"v(?P<version>.*)".to_string();
        let tag = "v1.2.3".to_owned();
        let version = parse_tag(&tag_pattern, &tag).unwrap();
        assert_eq!(version.0.to_string(), "1.2.3");
        assert_eq!(version.1, 1);
    }

    #[test]
    fn parse_tag_invalid_pattern() {
        let tag_pattern = r"v(?P<version>\d+\.\d+\.\d+".to_string();
        let tag = "v1.2.3".to_owned();
        let error = parse_tag(&tag_pattern, &tag).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InvalidTagPattern);
    }

    #[test]
    fn parse_tag_no_version() {
        let tag_pattern = r"v(?P<version>\d+\.\d+\.\d+)".to_string();
        let tag = "v1.2".to_owned();
        let error = parse_tag(&tag_pattern, &tag).unwrap_err();
        assert_eq!(error.kind, ErrorKind::NoVersionInTag);
    }
}

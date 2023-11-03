use clap::Parser;
use config::{is_config_available, parse_config_file, Config};
use error::{Error, ErrorKind};
use serde::Serialize;
use serde_json::to_string_pretty;
use source::SourceActions;
use std::{collections::HashMap, process::exit};
use version::{
    calculate_increment, increment_major, increment_minor, increment_patch, IncrementKind,
};

mod config;
mod error;
mod git;
mod parsing;
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

    /// All commits between the oldest tag and the one specified
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
    /// If at least one tag was created.
    tag_created: bool,
    /// New tags that were created.
    new_tags: Vec<String>,
    /// Information on the version bump of a scope.
    version_bumps: Vec<OutputVersionBump<'a>>,
    /// Commits that were skipped during the version bump due to pattern mismatch.
    skipped_commits: &'a Vec<String>,
    /// Error message if any.
    error: String,
}

/// Type for storing scope versions.
#[derive(Serialize, Debug, Clone)]
struct OutputVersionBump<'a> {
    /// Scope of the version.
    scope: String,
    /// Old version number before bump.
    old_version: String,
    /// New version number after bump.
    new_version: String,
    /// Kind of increment that was applied.
    increment_kind: &'a Option<IncrementKind>,
}

impl<'a> Output<'a> {
    /// Creates a new `Output` instance with the given `inputs`.
    fn new(inputs: &'a Args, config: Option<&'a Config>, skipped_commits: &'a Vec<String>) -> Self {
        Self {
            inputs,
            config,
            tag_created: false,
            new_tags: vec![],
            version_bumps: vec![],
            skipped_commits,
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

    // TODO: This will not work when we have more sources
    let source: source::SourceKind = match args.github_repo.clone() {
        Some(repo) => source::SourceKind::Github(source::github::GithubSource::new(
            &config,
            repo,
            validate_trailing_slash(&args.github_api_url),
            args.github_token.clone(),
        )),
        None => {
            if let Err(error) = git::verify_git() {
                print_error(error, &args, &output_format, Some(&config));
                exit(1);
            }
            source::SourceKind::Git(source::git::GitSource::new(&config))
        }
    };

    let commit_sha = match &args.commit_sha {
        Some(commit_sha) => commit_sha.clone(),
        None => match source.get_latest_commit_sha() {
            Ok(commit_sha) => commit_sha,
            Err(error) => {
                print_error(error, &args, &output_format, Some(&config));
                exit(1);
            }
        },
    };

    let ref_iterator = match source.get_ref_iterator(&commit_sha) {
        Ok(ref_iterator) => ref_iterator,
        Err(error) => {
            print_error(error, &args, &output_format, Some(&config));
            exit(1);
        }
    };

    let mut version_bumps: HashMap<String, Option<IncrementKind>> = HashMap::new();
    for scope in &config.version_scopes {
        version_bumps.insert(scope.clone(), None);
    }

    let mut skipped_commits_sha = vec![];
    let mut closest_tags = vec![];
    for r in ref_iterator {
        let r = match r {
            Ok(refs) => refs,
            Err(error) => {
                print_error(error, &args, &output_format, Some(&config));
                exit(1);
            }
        };

        if let Some(tags) = r.tags {
            closest_tags.reserve(tags.len());
            closest_tags.extend(tags);
        }

        if r.commit.is_none() {
            continue;
        }
        let commit = r.commit.unwrap();

        let commit_details = match &commit.details {
            Some(details) => details,
            None => {
                skipped_commits_sha.push(commit.sha.clone());

                if let OutputFormat::Text = output_format {
                    println!("commit '{}' does not match the commit pattern", commit.sha);
                }

                continue;
            }
        };

        let increment_kind = match calculate_increment(&commit, &config.bump_rules) {
            Some(increment_kind) => increment_kind,
            None => continue,
        };

        if let Some(prev_increment_kind) = version_bumps
            .get(commit_details.scope.as_ref().unwrap_or(&String::new()))
            .unwrap()
        {
            match prev_increment_kind {
                IncrementKind::Major => continue,
                IncrementKind::Minor => {
                    if increment_kind == IncrementKind::Major {
                        version_bumps.insert(
                            commit_details.scope.clone().unwrap_or_default(),
                            Some(increment_kind),
                        );
                    }
                }
                IncrementKind::Patch => {
                    if increment_kind != IncrementKind::Patch {
                        version_bumps.insert(
                            commit_details.scope.clone().unwrap_or_default(),
                            Some(increment_kind),
                        );
                    }
                }
            }
        } else {
            version_bumps.insert(
                commit_details.scope.clone().unwrap_or_default(),
                Some(increment_kind),
            );
        }
    }

    let version_bumps = version_bumps;
    let mut output = Output::new(&args, Some(&config), &skipped_commits_sha);

    let empty_scope = String::new();
    for tag in &mut closest_tags {
        let tag_details = tag.details.as_mut().unwrap();
        let scope = tag_details.scope.as_ref().unwrap_or(&empty_scope);

        let bump = version_bumps.get(scope).unwrap();

        let mut version_bump = OutputVersionBump {
            scope: scope.clone(),
            old_version: tag_details.version.to_string(),
            new_version: tag_details.version.to_string(),
            increment_kind: bump,
        };

        if bump.is_none() {
            output.version_bumps.push(version_bump);

            if let OutputFormat::Text = output_format {
                if scope.is_empty() {
                    println!("version bump for empty scope is not required");
                } else {
                    println!("version bump for scope {} is not required", scope);
                }
            }
            continue;
        }

        match bump.as_ref().unwrap() {
            IncrementKind::Major => {
                increment_major(&mut tag_details.version);
            }
            IncrementKind::Minor => {
                increment_minor(&mut tag_details.version);
            }
            IncrementKind::Patch => {
                increment_patch(&mut tag_details.version);
            }
        }
        version_bump.new_version = tag_details.version.to_string();
        if let OutputFormat::Text = output_format {
            if scope.is_empty() {
                println!(
                    "version bump for empty scope: {} -> {}",
                    version_bump.old_version, version_bump.new_version
                );
            } else {
                println!(
                    "version bump for scope {}: {} -> {}",
                    scope, version_bump.old_version, version_bump.new_version
                );
            }
        }
        output.version_bumps.push(version_bump.clone());

        if args.create_tag {
            let new_tag_name = tag
                .name
                .replace(&version_bump.old_version, &version_bump.new_version);
            let new_tag_message = &config.new_tag_message.replace("{scope}", scope);
            let new_tag_message = new_tag_message.replace("{version}", &version_bump.new_version);
            if let Err(error) = source.create_tag(&new_tag_name, &new_tag_message, &commit_sha) {
                print_error(error, &args, &output_format, Some(&config));
                exit(1);
            }
            output.tag_created = true;
            output.new_tags.push(new_tag_name.to_owned());

            if let OutputFormat::Text = output_format {
                println!("created tag {}", new_tag_name);
            }
        }
    }

    if let OutputFormat::Json = output_format {
        if let Ok(json_str) = to_string_pretty(&output) {
            println!("{}", json_str);
        } else {
            println!("could not serialize {:?}", output);
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
            let skipped_commits = vec![];
            let mut output = Output::new(inputs, config, &skipped_commits);
            output.error = format!("{}", error);
            if let Ok(json_str) = to_string_pretty(&output) {
                println!("{}", json_str);
            } else {
                println!("could not serialize {:?}", output);
            }
        }
    }
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

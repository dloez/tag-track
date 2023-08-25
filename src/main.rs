use clap::Parser;
use regex::Regex;
use semver::Version;
use serde::Serialize;
use serde_json::to_string_pretty;
use source::SourceActions;
use std::{collections::HashMap, process::exit};

mod error;
mod git;
mod source;
mod version;

const MAJOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf)!:";
const MINOR_REGEX_PATTERN: &str = r"^(feat|refactor|perf):";
const PATCH_REGEX_PATTERN: &str = r"^(fix|style):";

#[derive(Parser, Debug, Serialize, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Create git annotated tag from populated version.
    #[arg(long, default_value = "false", default_missing_value = "true")]
    create_tag: bool,

    // GitHUb repository identifier (owner/repo_name).
    // If pressent, this will use GitHub as the source to calculate a version bump.
    #[arg(long)]
    github_repo: Option<String>,

    // Token to authenticate  GitHub REST API calls.
    #[arg(long)]
    github_token: Option<String>,

    // All commits between the oldest closest tag and the one specified
    // by this SHA will be used to calculate the version bump. Useful when using
    // a remote reposity with different git history as the local repository.
    #[arg(long)]
    commit_sha: Option<String>,

    // Output format, possible values are: 'text', 'json'. Default value is 'text'.
    #[arg(long, default_value = "text", default_missing_value = "text")]
    output_format: String,
}

#[derive(Serialize, Debug)]
struct Output {
    inputs: Args,
    created_tag: bool,
    old_version: String,
    new_version: String,
    error: String,
}

impl Output {
    fn new(inputs: &Args) -> Self {
        Self {
            inputs: inputs.clone(),
            created_tag: false,
            old_version: "".to_owned(),
            new_version: "".to_owned(),
            error: "".to_owned(),
        }
    }
}

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
            let error = error::Error::new(error::ErrorKind::NotValidOutputFormat, Some(value));
            println!("{}", error);
            exit(1);
        }
    };

    if let Err(error) = git::verify_git() {
        return_error(output_format, error, &args);
        exit(1);
    }

    let current_commit_sha: String = match &args.commit_sha {
        Some(commit_sha) => commit_sha.to_owned(),
        None => match git::get_current_commit_sha() {
            Ok(current_commit) => current_commit,
            Err(error) => {
                return_error(output_format, error, &args);
                exit(1);
            }
        },
    };

    let mut source: source::SourceKind = match args.github_repo.clone() {
        Some(repo) => source::SourceKind::Github(source::github::GithubSource::new(
            repo,
            args.github_token.clone(),
        )),
        None => source::SourceKind::Git(source::git::GitSource::new()),
    };

    if let Err(error) = source.fetch_from_commit(&current_commit_sha) {
        return_error(output_format, error, &args);
        exit(1);
    };

    let source = source;
    let closest_tag = match source.get_closest_tag() {
        Ok(tag) => tag,
        Err(error) => {
            return_error(output_format, error, &args);
            exit(1);
        }
    };

    let mut version = match Version::parse(closest_tag) {
        Ok(version) => version,
        Err(error) => {
            return_error(output_format, error::Error::from(error), &args);
            exit(1);
        }
    };

    let commit_messages = match source.get_commit_messages() {
        Ok(commit_messages) => commit_messages,
        Err(error) => {
            return_error(output_format, error, &args);
            exit(1);
        }
    };

    let patterns: HashMap<version::IncrementKind, &'static str> = HashMap::from([
        (version::IncrementKind::Major, MAJOR_REGEX_PATTERN),
        (version::IncrementKind::Minor, MINOR_REGEX_PATTERN),
        (version::IncrementKind::Patch, PATCH_REGEX_PATTERN),
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
                    version::IncrementKind::Minor => increment_kind = Some(kind),
                    version::IncrementKind::Patch => {
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

    let mut output = Output::new(&args);
    output.old_version = version.to_string();

    let kind = increment_kind.unwrap();
    match kind {
        version::IncrementKind::Major => version::increment_major(&mut version),
        version::IncrementKind::Minor => version::increment_minor(&mut version),
        version::IncrementKind::Patch => version::increment_patch(&mut version),
    }
    output.new_version = version.to_string();

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
            return_error(output_format, error, &args);
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

fn return_error(output_format: OutputFormat, error: error::Error, inputs: &Args) {
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

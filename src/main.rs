use source::{github::GithubSource, SourceActions};

mod config;
mod error;
mod git;
mod parsing;
mod source;
mod version;

fn main() {
    let repo_id = String::from("dloez/tag-track-test");
    let api_url = String::from("https://api.github.com");
    let token = None;

    let mut config = config::Config::new();
    config.tag_pattern = String::from("(?<scope>.*)/(?<version>.*)");
    config.version_scopes = vec![String::from("example1"), String::from("example2")];

    let mut source = GithubSource::new(repo_id, api_url, token, &config);
    let commits = match source.get_commits("069faa2c784049fc9e9f97a57579059295dd8236") {
        Ok(commits) => commits,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    for commit in commits {
        println!("Commit: {:?}", commit);
    }

    println!("Tags: {:?}", source.get_closest_tags());
}

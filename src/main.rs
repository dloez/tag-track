use source::github::GithubSource;

mod config;
mod error;
mod git;
mod source;
mod version;

fn main() {
    let repo_id = String::from("dloez/tag-track-test");
    let api_url = String::from("https://api.github.com");
    let token = None;

    let mut config = config::Config::new();
    config.tag_pattern = String::from("(?<scope>.*)/(?<version>.*)");
    config.version_scopes = vec![String::from("example1"), String::from("example2s")];

    let source = GithubSource::new(repo_id, api_url, token, &config);
}

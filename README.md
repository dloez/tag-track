# Tag Track
Tag track is a version bump calculator and tracker that uses git tags to determine the current version and conventional commit messages to calculate the required bump.
Tag track has been designed with the following key points:
- Language agnostic. By using git tags to get the current version Tag track does not require any language integration. All projects that uses git can use Tag track.
- Usable in CI environments. Tag track can rely in other sources to get the git history information required to work, such as the usage of REST API like the one provided by GitHub. This makes Tag track work even in environments where the git history is partially available like in CI environments.
- Simple. All you need to do to run Tag track is download a prebuilt binary and run it. There are also instructions to built your own binary in case a prebuilt binary for your platform can't be found.

## Installation
To install Tag track, visit the `Installation` section of [the latest release](https://github.com/dloez/tag-track/releases/latest).

## Usage
Tag track needs to be executed inside a git working tree and requires at least one tag to work. To run tag track, run:
```sh
tag-track
```
This will use the latest closest tag and the commit messages between that tag and the current commit to calculate the version bump.

### Additional arguments
- `--create-tag`: Automatically create a git tag with the calculated version bump. If no version bump was calculated, the tag creation will be skipped.
- `--github-repo`: Use GitHub REST API instead of git history to calculate the version bump. The value should be `user-organization/repository-name`, for example `dloez/tag-track`.
- `--github-token`: GitHub token for authorizing the REST API calls. Useful when the target repository is private or when hitting the REST API rate limit without using authorized requests.
- `--commit-sha`: Commits between this SHA and the closest tag will be used to calculate the version bump. Useful in combination of remote sources such as GitHub REST API or if you do not want to use the current commit to calculate the version bump.
- `--output-format`: Format used to print the output. Useful for integrating Tag track with automatic systems. Possible values are `text` and `json`. Defaults to `text`.

### Configuration
Additionally, you can create a `track.yaml` or `track.yml` file to configure:

- Tag format. Use the `tag_pattern` to specify a Regex pattern to get the version from the tag name. For example, to get the version from a tag named `v1.0.0` use the pattern `v(.*)`. If not `tag_pattern` is specified, whole tag name will be used as version.

## Project stability and current status
Currently, Tag track has a high work-in-progress status, thus we are marking releases as pre-releases. There will be API changes and non-backward compatibility changes during this phase. Here is a list of features and improvements that we want to make before the release of Tag track `1.0.0`:

- [ ] Custom GitHub actions with improved API to use inside GitHub actions jobs.
- [ ] More remote sources such as GitLab, Gitea, etc.
- [ ] Custom rules to calculate version bumps based on conventional commit messages.
- [x] Support multiple tag formats. Currently only tags that follow the [semantic versioning 2.0](https://semver.org/) can be parsed.
- [ ] Support conventional commits message scopes and use the scopes to differentiate between different applications with different versions. This is our attempt to support monorepos.
- [ ] Improved usage of the git history when using remote sources. Currently if the argument `--create-tag` is not used with the use of a remote source like GitHub REST API, the git history is still required, which does not have sense.
- [ ] Improved error handling and user input control.

After the release of the version `1.0.0` we aim to not introduce any non-backward compatibility changes.

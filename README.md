# Tag Track
Tag track is a version bump calculator and tracker that uses git tags with (semantic versions)[https://semver.org] and (conventional commits)[https://www.conventionalcommits.org/en/v1.0.0] to calculate the required bump.
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

Refer to the [GitHub Action section](#github-action) for information on how to use Tag Track inside a GitHub actions workflow.

### Additional arguments
- `--create-tag`: Automatically create a git tag with the calculated version bump. If no version bump was calculated, the tag creation will be skipped.
- `--github-repo`: Use GitHub REST API instead of git history to calculate the version bump. The value should be `user-organization/repository-name`, for example `dloez/tag-track`.
- `--github-token`: GitHub token for authorizing the REST API calls. Useful when the target repository is private or when hitting the REST API rate limit without using authorized requests.
- `--commit-sha`: Commits between this SHA and the closest tag will be used to calculate the version bump. Useful in combination of remote sources such as GitHub REST API or if you do not want to use the current commit to calculate the version bump.
- `--output-format`: Format used to print the output. Useful for integrating Tag track with automatic systems. Possible values are `text` and `json`. Defaults to `text`.

### Configuration
Additionally, you can create a `track.yaml` or `track.yml` file to configure:

- Tag format. Use the `tag_pattern` field to specify a Regex pattern to get the version from the tag name. The pattern should contain naming capturing groups to capture the required fields. Read more about the required groups for this field in the [Naming capturing groups section](#naming-capturing-groups). The default pattern is `(?<version>.*)`.
- Commit pattern. Use the `commit_pattern` field to specify a Regex pattern to get the commit fields specified in the [conventional commit specification](https://www.conventionalcommits.org/en/v1.0.0). The pattern should contain naming capturing groups to capture the required fields. Read more about the required groups for this field in the [Naming capturing groups section](#naming-capturing-groups). The default pattern is `^(?<type>[a-zA-Z]*)(?<scope>\(.*\))?(?<breaking>!)?:(?<description>[\s\S]*)$`, this pattern tries to follow the [conventional commit specification](https://www.conventionalcommits.org/en/v1.0.0/#specification) as close **and simple** as possible while allowing to use custom types.
- Version bump rules. Use the `bump_rules` field to specify the rules used to calculate the version bump. If a rule condition is missing, that condition will not be evaluated. Each rule has the following fields:
  * `bump`: Version section to bump. Possible values are `major`, `minor`, and `patch`. Example: `bump: major` - will increase the `major` section of the semantic version if all rule conditions pass.
  * `types`: Condition - List of commit types. An `OR` operation will be used between list types. Example: `types: [feat, fix]` - will pass the condition if the commit type is `feat` or `fix`.
  * `scopes`: Condition - List of commit scopes. An `OR` operation will be used between list scopes. Example: `scopes: [api, cli]` - will pass the condition if the commit scope is `api` or `cli`.
  * `if_breaking_type`: Condition - If `true`, the rule will trigger if the Regex pattern specified in `commit_pattern` matches the `breaking` section of the commit message. Example: `if_breaking_type: true` - will pass the rule if commit type is `feat!` or `docs(api)!` and the default commit pattern is used.
  * `if_breaking_description`: Condition - If `true`, the rule will trigger if the strings `BREAKING CHANGE` or `BREAKING-CHANGE` are found in the commit description. Example: `if_breaking_description: false` - will not pass the rule if the commit description includes the string `BREAKING CHANGE`.

If multiple conditions are used in the same rule, an `AND` operation will be used between them. If multiple rules can be applied to the same commit, the biggest version change will be used (Major > Minor > Patch). Example of a configuration file with a rule that bumps the major section if the commit type is `feat` and matches the `breaking` section (there is the `!` in the commit type):
```yaml
bump_rules:
  - bump: major
    types: [feat]
    if_breaking_type: true
```
Example with the default valus for all configuration fields:
```yaml
tag_pattern: '(?<version>.*)'
bump_rules: '^(?<type>[a-zA-Z]*)(?<scope>\(.*\))?(?<breaking>!)?:(?<description>[\s\S]*)$'
    - bump: patch
      types: [fix, style]
    - bump: minor
      types: [feat, refactor, perf]
    - bump: major
      if_breaking_type: true
      if_breaking_description: true
```

## GitHub Action
We provide a custom GitHub action to easily call Tag Track in GitHub actions workflows. This action can download or compile the version of Tag Track based on the specified action version. This action has been tested in `Linux`, `MacOS`, and `Windows` hosted runners.

### Pre-requisites
In order for Tag Track to work, the repository needs to be checked out before calling Tag Tack.

### Inputs
- `create-tag`: Create a new tag with bumped version. Defaults to `false`.
- `push-tag`: Push the new tag to the repository if `create-tag` is used. Defaults to `false`.
- `github-repo`: Github repository in the format owner/repo. Defaults to the repository that triggers the action.
- `github-api-url`: Github API URL. Defaults to the value of the environment variable `GITHUB_API_URL`.
- `github-token`: Github token to authorize requests to GitHub REST API. Can cause rate limit to be increased.
- `commit-sha`: Commit SHA from where the version bump will be calculated. Defaults to the commit SHA that triggers the action.
- `compile`: Instead of downloading the binary, compile it from source. Defaults to `false`.
- `use-cache`: Save binary in cache to avoid downloading or compiling it every time. Defaults to `false`.
- `git-author-name`: Name of the author for tags created by Tag Track. Defaults to `github-actions`.
- `git-author-email`: Email of the author for tags created by Tag Track. Defaults to `github-actions@github.com`.

The action will automatically reset to previous git author information as soon as it finishes.

If you want to modify the different configuration properties, create a `track.yaml` or `track.yml` file in the root of the repository. Refer to the [Configuration section](#configuration) for more information.

### Outputs
- `tag-created`: `true` if a new tag was created.
- `new-tag`: Tag name if a new tag was created.
- `old-version`: Old version before calculating the version bump.
- `new-version`: New version after calculating the version bump.
- `skipped-commits`: List of commits that were skipped because they don't match the commit pattern.
- `error`: Error message if something went wrong.

The action will create notices to inform the user about the version bump and if a new tag was created and pushed. In case there are any skipped commits, a new warning for each of the commits will be created.

### Examples
The version shown on these examples are only for demostration pourposes, use [the latest stable release version](https://github.com/dloez/tag-track/releases/latest) in your workflows. If you want to use any other version that does not have a release, use the compile mode.

- Checkout repository, calculate version bump, and create and push a new tag with the bumped version. Use cache to save binary and avoid downloading it every time:
```yaml
- name: Checkout
  uses: actions/checkout@v4
- name: Tag Track
  uses: dloez/tag-track@v1.0.0
  with:
    create-tag: true
    push-tag: true
    use-cache: true
```

- Checkout repository, calculate version bump, and create a new tag with the bumped version. Use cache to save binary and avoid downloading it every time. Instead of using Tag Track to push the image, push it manually:
```yaml
- name: Checkout
  uses: actions/checkout@v4
- name: Tag Track
  uses: dloez/tag-track@v1.0.0
  with:
    create-tag: true
    use-cache: true
- name: Push tags
  run: git push origin --tags
```

- Checkout repository, calculate version bump, and create and push a new tag with the bumped version. Instead of using a release, use the main branch and the compile mode with cache to avoid compiling it every time:
```yaml
- name: Checkout
  uses: actions/checkout@v4
- name: Tag Track
  uses: dloez/tag-track@v1.0.0
  with:
    create-tag: true
    compile: true
    use-cache: true
```

## Naming capturing groups
Tag track uses [naming capturing groups](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Regular_expressions/Named_capturing_group) to get the required information from tags and commits. The naming capturing groups allows the user to define custom patterns (even with different field ordering) and still get the required information. These are the naming capturing groups used by Tag track:
- `tag_pattern`:
  * `version`: captures the version that should follow the [semantic versioning 2.0 specfification](https://semver.org). This group is required.
- `commit_pattern`: The following groups corresponds to the different fields in specified in the [conventional commit specification](https://www.conventionalcommits.org/en/v1.0.0):
  * `type`: captures the commit type. This group is required.
  * `scope`: captures the commit scope.
  * `breaking`: captures the breaking change indicator.
  * `description`: captures the commit description. This group is required.

## Project stability and current status
Currently, Tag track has a high work-in-progress status, thus we are marking releases as pre-releases. There will be API changes and non-backward compatibility changes during this phase. Here is a list of features and improvements that we want to make before the release of Tag track `1.0.0`:

- [x] Custom GitHub actions with improved API to use inside GitHub actions jobs.
- [ ] More remote sources such as GitLab, Gitea, etc.
- [x] Custom rules to calculate version bumps based on conventional commit messages.
- [x] Support multiple tag formats. Currently only tags that follow the [semantic versioning 2.0](https://semver.org) can be parsed.
- [x] Support multiple commit formats.
- [ ] Use the commit scope to differentiate between different applications with different versions. This is our attempt to support monorepos.
- [ ] Improved usage of the git history when using remote sources.
- [ ] Improved error handling and user input control.

After the release of the version `1.0.0` we aim to not introduce any non-backward compatibility changes.

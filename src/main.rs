use std::{
    io::Error,
    io::ErrorKind,
    process::{exit, Command},
};

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

fn main() {
    if let Err(error) = verify_git() {
        println!("Git verification not passed, error: {}", error);
        exit(1);
    }

    let tag = get_closest_tag().expect("failed to get closest tag");
    println!("tag: {}", tag);
    let tag_commit_sha = get_tag_commit_sha(&tag).expect("failed to get tag commit sha");
    println!("tag {} - commit: {}", tag, tag_commit_sha);
    let commits_messages = get_commit_messages(&tag_commit_sha, &String::from("HEAD"))
        .expect("could not get commit messages");
    println!("{:?}", commits_messages);
}

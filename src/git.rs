use std::process::Command;

use crate::error::{Error, ErrorKind};

pub fn verify_git() -> Result<(), Error> {
    if let Err(error) = Command::new("git").arg("--version").output() {
        return Err(Error::new(ErrorKind::MissingGit, Some(&error.to_string())));
    }

    let output_result = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => return Err(Error::new(
            ErrorKind::GenericCommandFailed,
            Some(&error.to_string()),
        )),
    };

    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::NotGitWorkingTree,
            None
        ))
    }

    Ok(())
}

pub fn get_current_commit() -> Result<String, Error> {
    let output_result = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => return Err(Error::new(
            ErrorKind::GenericCommandFailed,
            Some(&error.to_string()),
        )),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get current commit: {} - error code: {}",
                stderr,
                output.status.code().unwrap()
            )),
        ));
    }    

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

pub fn get_closest_tag() -> Result<String, Error> {
    let output_result = Command::new("git")
        .arg("describe")
        .arg("--abbrev=0")
        .arg("--tags")
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => return Err(Error::new(
            ErrorKind::GenericCommandFailed,
            Some(&error.to_string()),
        )),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get closest tag: {} - code: {}",
                stderr,
                output.status.code().unwrap()
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

pub fn get_tag_commit_sha(tag: &String) -> Result<String, Error> {
    let output_result = Command::new("git")
        .arg("rev-list")
        .args(["-n", "1"])
        .arg(tag)
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => return Err(Error::new(
            ErrorKind::GenericCommandFailed,
            Some(&error.to_string()),
        )),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get tag commit sha: {} - code: {}",
                stderr,
                output.status.code().unwrap()
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(String::from(stdout.strip_suffix('\n').unwrap()))
}

pub fn get_commit_messages(from_commit: &String, until_commit: &String) -> Result<Vec<String>, Error> {
    let output_result = Command::new("git")
        .arg("log")
        .arg("--format=%s")
        .arg(from_commit)
        .arg(until_commit)
        .output();

    let output = match output_result {
        Ok(output) => output,
        Err(error) => return Err(Error::new(
            ErrorKind::GenericCommandFailed,
            Some(&error.to_string()),
        )),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::new(
            ErrorKind::Other,
            Some(&format!(
                "can not get commit between '{}' and '{}', stderr: {} - code: {}",
                from_commit,
                until_commit,
                stderr,
                output.status.code().unwrap()
            )),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut iterator = stdout.lines().map(|s| s.to_owned());
    let mut commit_messages = Vec::new();
    while let Some(line) = iterator.next() {
        if iterator.clone().next().is_some() {
            commit_messages.push(line);
        }
    }
    Ok(commit_messages)
}

// fn create_tag(tag: &String, tag_message: &String) -> Result<(), Error>{
//     let output = Command::new("git")
//         .arg("tag")
//         .args(["-a", tag])
//         .args(["-m", tag_message])
//         .output()?;

//     if !output.status.success() {
//         let stderr = String::from_utf8_lossy(&output.stderr).to_string();
//         return Err(Error::new(
//             ErrorKind::Other,
//             format!(
//                 "can not create tag '{}', stderr: {} - code: {}",
//                 tag,
//                 stderr,
//                 output.status.code().unwrap()
//             ),
//         ));
//     }

//     Ok(())
// }

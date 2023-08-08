use std::process::Command;

use crate::error::ErrorKind;

pub fn verify_git<'a>() -> Result<bool, ErrorKind<'a>> {
    if let Err(_) = Command::new("git").arg("--version").output() {
        return Err(ErrorKind::MissingGit)
    }

    if true {
        return Err(ErrorKind::GenericCommandFailed("example"))
    }

    // let output = Command::new("git")
    //     .arg("rev-parse")
    //     .arg("--is-inside-work-tree")
    //     .output()?;

    // if !output.status.success() {
    //     return Err(Error::new(ErrorKind::Other, "not in a git working tree"));
    // }
    Ok(true)
}

// fn get_closest_tag() -> Result<String, Error> {
//     let output = Command::new("git")
//         .arg("describe")
//         .arg("--abbrev=0")
//         .arg("--tags")
//         .output()?;

//     if !output.status.success() {
//         let stderr = String::from_utf8_lossy(&output.stderr).to_string();
//         return Err(Error::new(
//             ErrorKind::Other,
//             format!(
//                 "can not get closest tag: {} - code: {}",
//                 stderr,
//                 output.status.code().unwrap()
//             ),
//         ));
//     }

//     let stdout = String::from_utf8_lossy(&output.stdout).to_string();
//     Ok(String::from(stdout.strip_suffix('\n').unwrap()))
// }

// fn get_tag_commit_sha(tag: &String) -> Result<String, Error> {
//     let output = Command::new("git")
//         .arg("rev-list")
//         .args(["-n", "1"])
//         .arg(tag)
//         .output()?;

//     if !output.status.success() {
//         let stderr = String::from_utf8_lossy(&output.stderr).to_string();
//         return Err(Error::new(
//             ErrorKind::Other,
//             format!(
//                 "can not get tag commit sha: {} - code: {}",
//                 stderr,
//                 output.status.code().unwrap()
//             ),
//         ));
//     }

//     let stdout = String::from_utf8_lossy(&output.stdout).to_string();
//     Ok(String::from(stdout.strip_suffix('\n').unwrap()))
// }

// fn get_commit_messages(from_commit: &String, to_commit: &String) -> Result<Vec<String>, Error> {
//     let output = Command::new("git")
//         .arg("log")
//         .arg("--format=%s")
//         .arg(from_commit)
//         .arg(to_commit)
//         .output()?;

//     if !output.status.success() {
//         let stderr = String::from_utf8_lossy(&output.stderr).to_string();
//         return Err(Error::new(
//             ErrorKind::Other,
//             format!(
//                 "can not get commit between '{}' and '{}', stderr: {} - code: {}",
//                 from_commit,
//                 to_commit,
//                 stderr,
//                 output.status.code().unwrap()
//             ),
//         ));
//     }

//     let stdout = String::from_utf8_lossy(&output.stdout).to_string();
//     Ok(stdout.lines().map(|s| s.to_owned()).collect())
// }

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

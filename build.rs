use std::{io::{Error, ErrorKind}, process::Command};

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

fn main() {
    match get_closest_tag() {
        Err(_) => println!("cargo:rustc-env=CARGO_PKG_VERSION=0.1.0"),
        Ok(tag) => println!("cargo:rustc-env=CARGO_PKG_VERSION={}", tag),
    };
}

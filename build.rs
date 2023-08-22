use std::{
    env,
    io::{Error, ErrorKind},
    process::Command,
};

fn get_version_from_closest_tag() -> Result<String, Error> {
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

fn get_version_from_env(env_var_name: &str) -> Option<String> {
    match env::var(env_var_name) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

fn print_cargo_version(version: String) {
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", version)
}

static ENV_VAR_NAME: &str = "TAG_TRACK_VERSION";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git");
    println!("cargo:rerun-if-env-changed={}", ENV_VAR_NAME);

    if let Some(version) = get_version_from_env(ENV_VAR_NAME) {
        print_cargo_version(version);
        return;
    }

    if let Ok(version) = get_version_from_closest_tag() {
        print_cargo_version(version);
    }
}

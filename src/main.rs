use std::{process::{Command, exit}, io::Error, io::ErrorKind};


fn verify_git() -> Result<bool, Error> {
    Command::new("git")
        .arg("--version")
        .output()?;

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()?;
    
    if !output.status.success() {
        return Err(Error::new(ErrorKind::Other, "Not in a git working tree"));
    }
    Ok(true)
}

fn get_closes_tag() -> Result<String, Error> {
    let output = Command::new("git")
        .arg("describe")
        .arg("--abrev=0")
        .arg("--tags")
        .output()?;
}


fn main() {
    match verify_git() {
        Ok(_) => println!("Git verification passed"),
        Err(error) => {
            println!("Git verification not passed, error: {}", error);
            exit(1);
        }   
    }
}


// Obtain closest tag: git describe --abbrev=0 --tags
// Obtain commit sha from the latest tag: git rev-list -n 1 0.2.2
// Read commit messages from tag commit to HEAD
// Increment version based on iterated commit messages

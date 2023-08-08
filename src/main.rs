mod error;
mod git;
mod source;

fn main() {
    // let output = git::verify_git();

    match git::verify_git() {
        Ok(_) => todo!(),
        Err(error) => println!("{:?}", error),
    }
}

mod source;
mod git;
mod error;

fn main() {
    // let output = git::verify_git();

    match git::verify_git() {
        Ok(_) => todo!(),
        Err(error) => println!("{:?}", error),
    }

    println!("yikes");
}

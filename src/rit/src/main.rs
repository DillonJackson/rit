mod utility;
mod obj_database;
use clap::Parser;
use std::io;

//global variables
const DIRECTORY_PATH: &str = ".rit";

// remove .rit folder and its contents
fn rit_remove() -> io::Result<()> {
    utility::repo_remove(DIRECTORY_PATH)?;
    Ok(())
}

// initialize .rit folder
fn rit_init() -> io::Result<()> {
    utility::init_file_structure()?;
    Ok(())
}

/// A simple CLI application.
#[derive(Parser)]
struct Cli {
    /// An input argument
    input: String,
}


// **Usages**
// cargo run init                -- runs the init command
// cargo run "repo remove"       -- runs the repo remove command

fn main() -> io::Result<()> {

    // Parsing command line arguments
    let cli = Cli::parse();
    
    // calls commands
    match cli.input.as_str() {
        "init" => rit_init()?,
        "repo remove" => rit_remove()?,
        "hash" =>  match obj_database::store_data("utility.rs") {
            Ok(key) => println!("Key: {}", key),
            Err(e) => println!("Error: {}", e),
        }
        _ => println!("Unknown command: {}", cli.input),
    }
    Ok(())
}
mod utility;
use clap::Parser;
// use std::env;
use std::io;

//global variables
const DIRECTORY_PATH: &str = ".rit";

/// test functions 
fn test1() {
    println!("Hello from Test1");
}

fn test2() {
    println!("Hello from Test2");
}


fn rit_remove() -> io::Result<()> {

 utility::repo_remove(DIRECTORY_PATH)?;
 Ok(())
}

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

fn main() -> io::Result<()> {
    // Parsing command line arguments
    let cli = Cli::parse();
    
    // Call function
    match cli.input.as_str() {
        "test" => test1(),
        "test2" => test2(),
        "init" => rit_init()?,
        "repo remove" => rit_remove()?,
        _ => println!("Unknown command: {}", cli.input),
    }
    Ok(())
}
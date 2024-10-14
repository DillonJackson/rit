mod utility;
mod obj_database;
mod args;
mod constants;
mod index;


use args::{RitArgs, Commands};
use std::io;
use std::path::Path;
use clap::Parser;
use constants::DIRECTORY_PATH;

// 100644 for normal files.
// 100755 for executable files.
// 120000 for symbolic links.

//global variables


// remove .rit folder and its contents
fn rit_remove() -> io::Result<()> {
    utility::repo_remove(DIRECTORY_PATH)?;
    Ok(())
}

// initialize .rit folder
fn rit_init() -> io::Result<()> {
    // Check if the repository is already initialized
    if Path::new(DIRECTORY_PATH).exists() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Repository already initialized."));
    }
    
    utility::init_file_structure()?;
    println!("Repository initialized at {}.", DIRECTORY_PATH);
    Ok(())
}

// Helper function to check if the repository is initialized
fn check_repo_initialized() -> io::Result<()> {
    if !Path::new(DIRECTORY_PATH).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Repository not initialized. Please run `rit init` first."));
    }
    Ok(())
}

// **Usages**
// cargo run                     -- returns the help message   
// cargo run init                -- runs the init command
// cargo run remove              -- runs the repo remove command
// cargo run hash-object         -- runs the repo remove command

fn main() -> io::Result<()> {

    // Parsing command line arguments
    let args = RitArgs::parse();
    
    // calls commands
    match args.command {
        Commands::Init => {
            rit_init()?;
        },
        Commands::Remove => {
            check_repo_initialized()?;
            rit_remove()?;
        },
        Commands::HashObject(hash_args) => {
            check_repo_initialized()?;
            obj_database::store_data(&hash_args.file)?;
        }
    }

    Ok(())
}
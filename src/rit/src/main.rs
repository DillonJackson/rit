mod utility;
mod obj_database;
mod args;
mod constants;
mod index;
mod staging;
mod repo;

use repo::{rit_init, rit_remove, check_repo_initialized};
use args::{RitArgs, Commands};
use clap::Parser;
use std::io;

// 100644 for normal files.
// 100755 for executable files.
// 120000 for symbolic links.

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
            obj_database::store_file(&hash_args.file)?;
        }
        Commands::Blob(hash_args) => {
            check_repo_initialized()?;
            let data = obj_database::get_data(&hash_args.key)?;
            println!("{}", String::from_utf8_lossy(&data));
        }
        Commands::Add(add_args) => {
            check_repo_initialized()?;
            staging::add_file_to_staging(&add_args.file)?;
        }
    }

    Ok(())
}
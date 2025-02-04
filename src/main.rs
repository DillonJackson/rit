mod utility;
mod database;
mod args;
mod constants;
mod index;
mod staging;
mod repo;
mod compression;
mod hash;
mod commit;
mod tree;
mod branches;


use repo::{rit_init, rit_remove, check_repo_initialized};
use args::{RitArgs, Commands};
use clap::Parser;
use std::io;
use crate::index::get_status;

// 100644 for normal files.
// 100755 for executable files.
// 120000 for symbolic links.

// **Usages**
// cargo run                     -- returns the help message   
// cargo run init                -- runs the init command
// cargo run remove              -- runs the repo remove command
// cargo run hash-object         -- runs the remove command
// cargo run add                 -- runs the repo add command

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
            let key = database::store_file(&hash_args.file)?;
            println!("{}", key);
        }
        Commands::CatFile(cat_args) => {
            check_repo_initialized()?;
            let (_, _, data) = database::get_data(&cat_args.key)?;
            println!("{}", String::from_utf8_lossy(&data));
        },
        Commands::Blob(hash_args) => {
            check_repo_initialized()?;
            let (object_type, _, data) = database::get_data(&hash_args.key)?;
            if (object_type != constants::BLOB) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Object is not a blob"));
            }
            println!("{}", String::from_utf8_lossy(&data));
        },
        Commands::Add(add_args) => {
            check_repo_initialized()?;
            staging::add_file_to_staging(&add_args.file)?;
        },
        Commands::LsTree(hash_args) => {
            check_repo_initialized()?;
            let entries = tree::read_tree(&hash_args.key)?;
            for entry in entries {
                // Print each entry in the format: "<mode> <type> <hash>\t<name>"
                println!(
                    "{:06o} {}\t{}\t{}",
                    entry.mode,
                    entry.object_type,
                    entry.hash,
                    entry.name
                );
            }
        },
        Commands::Commit(commit_args) => {
            check_repo_initialized()?;
            let commit_hash = commit::commit(&commit_args.message, &commit_args.committer)?;
            println!("{}", commit_hash);
        },
        Commands::Status => {
            check_repo_initialized()?;
            get_status();
        }
    }

    Ok(())
}
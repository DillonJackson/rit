use crate::index;
use crate::utility;
use crate::constants::DIRECTORY_PATH;
use crate::database;
use crate::branches;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::fs;



// remove .rit folder and its contents
pub fn rit_remove() -> Result<()> {
    // Ask for user confirmation before removing the repository
    println!("Are you sure you want to remove the repository? (yes/no): ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "yes" {
        utility::repo_remove(DIRECTORY_PATH)?;
    }
    Ok(())
}

// initialize .rit folder
pub fn rit_init() -> Result<()> {
    // Get the path
    let path = Path::new(DIRECTORY_PATH);
    
    // Check if the repository is already initialized
    if path.exists() {
        return Err(Error::new(ErrorKind::AlreadyExists, "Repository already initialized."));
    }
    
    // Create the directory
    fs::create_dir_all(path)?;

    // Create the repository structure
    // utility::init_file_structure()?;
    database::create_object_database()?;
    index::create_index()?;
    branches::init_branches()?;

    println!("Repository initialized at {}.", DIRECTORY_PATH);
    Ok(())
}

// Helper function to check if the repository is initialized
pub fn check_repo_initialized() -> Result<()> {
    if !Path::new(DIRECTORY_PATH).exists() {
        return Err(Error::new(ErrorKind::NotFound, "Repository not initialized. Please run `rit init` first."));
    }
    Ok(())
}
use crate::index;
use crate::obj_database;
use std::io::{Error, ErrorKind};


pub fn add_file_to_staging(file_path: &str) -> Result<(), Error> {
    // Ensure the file exists
    if !std::path::Path::new(file_path).exists() {
        return Err(Error::new(ErrorKind::NotFound, "File not found"));
    }
    
    // Store the file in the object database
    let blob_hash = obj_database::store_file(file_path)?;
    
    // Check if the file is already in the latest commit

    // Add the file to the index
    index::add_to_index(file_path, &blob_hash)
}
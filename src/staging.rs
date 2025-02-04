use crate::index;
use crate::database;
use crate::commit;
use crate::index::IndexEntry;
use std::io::{Error, ErrorKind};


pub fn add_file_to_staging(file_path: &str) -> Result<(), Error> {
    // Ensure the file exists
    if !std::path::Path::new(file_path).exists() {
        return Err(Error::new(ErrorKind::NotFound, "File not found"));
    }
    
    // Store the file in the object database
    let blob_hash = database::store_file(file_path)?;
    
    // Check if the file is already in the latest commit

    // Add the file to the index
    index::add_to_index(file_path, &blob_hash)
}

pub fn get_staged_entries() -> std::io::Result<Vec<IndexEntry>> {
    index::load_index()
}
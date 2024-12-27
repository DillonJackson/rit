use json::object;

use crate::constants::{DIRECTORY_PATH, OBJECTS_DIR, BLOB};
// use crate::utility::{create_directory, open_file};
use crate::compression::{compress_data, uncompress_data};
use crate::hash::{hash_data};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::io::Write;

pub fn store_data(data: &[u8], object_type: &str) -> io::Result<String> {
    // Create metadata for the object
    let metadata = format!("{} {}\0", object_type, data.len());

    // Concatenate the metadata and data
    let mut object = metadata.into_bytes();
    object.extend_from_slice(data);

    //hash the data to obtain the key
    let key = hash_data(&object)?;

    // Get the path to the object file
    let object_path = get_object_path(&key);

    // Check if the file already exists
    if object_path.exists() {
        return Ok(key);
    }

    // Ensure the parent directory exists before writing the file
    if let Some(parent_dir) = object_path.parent() {
        fs::create_dir_all(parent_dir)?; // Create the directory if it doesn't exist
    }

    // Compress the data before writing it to the file
    let object = compress_data(&object)?;

    // Write the data to the file in the object database
    let mut file = File::create(object_path)?; // Create the file
    file.write_all(&object)?; // Write the data to the file
    file.flush()?; // Ensure all data is written to disk

    // Return the key
    Ok(key)
}

pub fn store_file(file_path: &str) -> io::Result<String> {
    // Open the file in read-only mode and read its contents into a buffer
    let mut buffer = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut buffer)?;

    // Store the data in the object database
    store_data(&buffer, BLOB)
}

pub fn get_data(key: &str) -> io::Result<(String, usize, Vec<u8>)> {
    let file_path = get_object_path(key);
    if file_path.exists() {
        // Open the file and read its contents into a buffer
        let mut buffer = Vec::new();
        let mut file = File::open(file_path)?;
        file.read_to_end(&mut buffer)?;
        let data = uncompress_data(&buffer)?;
        let (object_type, object_size, object_data) = parse_metadata_and_data(&data)?;
        Ok((object_type.to_string(), object_size, object_data.to_vec()))
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Object not found"))
    }
}

fn parse_metadata_and_data(data: &[u8]) -> io::Result<(&str, usize, &[u8])> {
    // Find the position of the first space character in the data
    let first_space = data
        .iter()
        .position(|&x| x == b' ')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid metadata: no space"))?;

    // Find the position of the null character in the data
    let null_char = data
        .iter()
        .position(|&x| x == b'\0')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid metadata: no null terminator"))?;

    // Extract the object type from the data
    let object_type = std::str::from_utf8(&data[..first_space])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid object type"))?;

    // Extract the object size from the data
    let object_size_str = std::str::from_utf8(&data[first_space + 1..null_char])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid object size"))?;
    let object_size: usize = object_size_str
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Object size is not a valid number"))?;

    // Extract the object data from the data
    let object_data = &data[null_char + 1..];

    Ok((object_type, object_size, object_data))
}

pub fn delete_data(key: &str) -> io::Result<()> {
    let file_path = get_object_path(key);
    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

// HELPERS
// Returns the path to a specific object based on the key
fn get_object_path(key: &str) -> PathBuf {
    let sub_dir_name: String = key.chars().take(2).collect();
    let filename: String = key.chars().skip(2).collect();
    Path::new(DIRECTORY_PATH)
        .join(OBJECTS_DIR)
        .join(sub_dir_name)
        .join(filename)
}

// Returns the path to the object database directory
fn get_object_database_path() -> PathBuf {
    Path::new(DIRECTORY_PATH).join(OBJECTS_DIR)
}

// Creates the object database directory
pub fn create_object_database() -> io::Result<()> {
    let result = get_object_database_path();
    fs::create_dir_all(&result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::env;
    use std::path::PathBuf;

    // Helper function to create a temp directory and set it as the current working directory
    fn setup_test_env() -> PathBuf {
        // Create a temporary directory for the test
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let original_dir = env::current_dir().unwrap();

        // Change the current working directory to the temp directory
        env::set_current_dir(&temp_dir_path).unwrap();

        // Return the original directory so we can restore it after the test
        original_dir
    }

    #[test]
    fn test_store_and_get_data() {
        let original_dir = setup_test_env(); // Switch to temp directory

        // Perform the test
        let data = b"example data";
        let key = store_data(data, BLOB).unwrap();

        // Check if the data was stored and can be retrieved correctly
        let (object_type, object_size, object_data) = get_data(&key).unwrap();
        assert_eq!(object_data, data);
        assert_eq!(object_type, BLOB);
        assert_eq!(object_size, data.len());

        // Restore the original directory
        env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_store_and_delete_data() {
        let original_dir = setup_test_env();

        let data = b"example data";
        let key = store_data(data, BLOB).unwrap();

        let (object_type, object_size, object_data) = get_data(&key).unwrap();
        print!("{:?}", object_data);
        assert_eq!(object_data, data);
        print!("{:?}", object_type);
        assert_eq!(object_type, BLOB);
        print!("{:?}", object_size);
        assert_eq!(object_size, data.len());

        delete_data(&key).unwrap();
        assert!(get_data(&key).is_err());

        env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_store_file() {
        let original_dir = setup_test_env();

        // Create a temporary file with some content
        let file_path = PathBuf::from("test_file.txt");
        let file_data = b"file data";
        fs::write(&file_path, file_data).unwrap();

        // Store the file in the object database
        let key = store_file(file_path.to_str().unwrap()).unwrap();

        // Retrieve and verify the stored content
        let (object_type, object_size, object_data) = get_data(&key).unwrap();
        assert_eq!(object_data, file_data);
        assert_eq!(object_type, BLOB);
        assert_eq!(object_size, file_data.len());

        env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_data_not_found() {
        let original_dir = setup_test_env();

        let non_existent_key = "nonexistentkey1234567890";
        assert!(get_data(non_existent_key).is_err());

        env::set_current_dir(&original_dir).unwrap();
    }
}

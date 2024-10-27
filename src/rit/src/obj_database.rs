use crate::constants::{DIRECTORY_PATH, OBJECTS_DIR};
// use crate::utility::{create_directory, open_file};
use crate::compression::{compress_data, uncompress_data};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::io::Write;

pub fn store_data(data: &[u8]) -> io::Result<String> {
    //hash the data to obtain the key
    let key = hash_data(&data)?;

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
    let data = compress_data(data)?;

    // Write the data to the file in the object database
    let mut file = File::create(object_path)?; // Create the file
    file.write_all(&data)?; // Write the data to the file
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
    store_data(&buffer)

    // // Open the file in read-only mode
    // let buffer = open_file(&file_path)?;
    // //hash the file to obtain the key
    // let key = match hash_file(&buffer) {
    //     Ok(hash_value) => hash_value,
    //     Err(e) => {
    //         eprintln!("Error calculating hash: {}", e);
    //         return Err(e);
    //     },
    // };

    // // Print the stored hash value
    // println!("SHA-256 hash: {}", key);

    // //directory filename
    // let sub_dir_name: String = key.chars().take(2).collect();
    // // println!("SHA-256 hash first 2 char: {}", sub_dir_name);

    // //file filename
    // let filename: String = key.chars().skip(2).collect();
    // // println!("SHA-256 hash first 2 char: {}", filename);

    // // Check if the file already exists
    // let file_path = get_object_path(&key);
    // if std::path::Path::new(&file_path).exists() {
    //     println!("File already exists.");
    //     return Ok(key);
    // }

    // //creates the file
    // let result: String = format!("{}/{}", ".rit/objects", sub_dir_name);
    // let result_str: &str = &result;
    // // println!("dir {}", result_str);
    // create_file(&result_str, &filename, Some(&buffer))?;
    // Ok(key)
}

pub fn get_data(key: &str) -> io::Result<Vec<u8>> {
    let file_path = get_object_path(key);
    if file_path.exists() {
        // Open the file and read its contents into a buffer
        let mut buffer = Vec::new();
        let mut file = File::open(file_path)?;
        file.read_to_end(&mut buffer)?;
        uncompress_data(&buffer)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Object not found"))
    }

    // let sub_dir_name: String = key.chars().take(2).collect();
    // let filename: String = key.chars().skip(2).collect();
    // let file_path: String = format!(".rit/objects/{}/{}", sub_dir_name, filename);
    // let buffer = open_file(&file_path)?;
    // match uncompress_data(&buffer) {
    //     Ok(data) => {
    //         if let Ok(string_data) = String::from_utf8(data.clone()) {
    //             println!("Decompressed data: {}", string_data);
    //         } else {
    //             println!("Decompressed data (binary): {:?}", data);
    //         }
    //         Ok(data)
    //     },
    //     Err(e) => Err(e),
    // }
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

// hash the file, then returns the key of the file
fn hash_data(buffer: &[u8]) -> io::Result<String> {
    // Create a new SHA-256 hasher
    let mut hasher = Sha256::new();

    // Update the hasher with the file contents
    hasher.update(buffer);

    // Finalize the hash and convert it to a hexadecimal string
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
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
        let key = store_data(data).unwrap();

        // Check if the data was stored and can be retrieved correctly
        let retrieved_data = get_data(&key).unwrap();
        assert_eq!(retrieved_data, data);

        // Restore the original directory
        env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_store_and_delete_data() {
        let original_dir = setup_test_env();

        let data = b"example data";
        let key = store_data(data).unwrap();

        let retrieved_data = get_data(&key).unwrap();
        assert_eq!(retrieved_data, data);

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
        let retrieved_data = get_data(&key).unwrap();
        assert_eq!(retrieved_data, file_data);

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

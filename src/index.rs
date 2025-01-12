// This file will contain the methods to interact with the index file.
// What is stored in the index file?
// The index file stores the file name, the hash value of the file, and the file path.


use crate::constants::{DIRECTORY_PATH, INDEX_FILE,SOURCE_PATH};
use crate::database::store_temporary;
use crate::tree::{self, convert_tree_entry_to_hashmap};
use std::collections::HashMap;
use std::fs::{File};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::ops::Index;
use std::path::{Path, PathBuf};
use crate::hash::{hash_data};
use std::fs;
use tempdir::TempDir;
use crate::branches::{self, get_current_branch_commit_hash, get_current_tree_from_commit_hash};
use crate::staging;
use colored::Colorize;


#[derive(Debug, Clone, PartialEq)]
pub struct IndexEntry {
    pub mode: u32,
    pub blob_hash: String,
    pub path: String
}

pub fn get_index_path() -> PathBuf {
    Path::new(DIRECTORY_PATH).join(INDEX_FILE)
}

// This function will add the file to the index file.
pub fn create_index() -> io::Result<()> {
    let index_path = get_index_path();
    if !index_path.exists() {
        File::create(&index_path)?;
    }
    Ok(())
}

// This function will add the file to the index file.
pub fn add_to_index(file_path: &str, blob_hash: &str) -> io::Result<()> {
    // TODO trim off the ./ and validate the file path to ensure is in your repo
    let index = load_index()?;

    let mut index_map: HashMap<String, IndexEntry> = index.into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect();

    let entry = index_map.entry(file_path.to_string()).or_insert(IndexEntry {
        mode: 0o100644,
        blob_hash: blob_hash.to_string(),
        path: file_path.to_string(),
    });
    entry.blob_hash = blob_hash.to_string();

    let index: Vec<IndexEntry> = index_map.into_values().collect();
    save_index(&index)?;

    Ok(())
}

pub fn bulk_add_to_index(entries: &[(&str, &str)]) -> io::Result<()> {
    // TODO trim off the ./ and validate the file path to ensure is in your repo
    let index = load_index()?;

    let mut index_map: HashMap<String, IndexEntry> = index.into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect();

    for (file_path, blob_hash) in entries {
        let entry = index_map.entry(file_path.to_string()).or_insert(IndexEntry {
            mode: 0o100644,
            blob_hash: blob_hash.to_string(),
            path: file_path.to_string(),
        });
        entry.blob_hash = blob_hash.to_string();
    }

    let index: Vec<IndexEntry> = index_map.into_values().collect();
    save_index(&index)?;

    Ok(())
}


// This function will read the index file and return the entries.
pub fn load_index() -> io::Result<Vec<IndexEntry>> {
    let index_path = get_index_path();
    let mut entries = Vec::new();

    if !index_path.exists() {
        return Ok(entries);
    }

    let file = File::open(&index_path)?;
    let mut reader = BufReader::new(file);

    while let Ok(entry) = read_index_entry(&mut reader) {
        entries.push(entry);
    }

    // println!("{:?}", entries);
    Ok(entries)
}

// This function will save the index entries to the index file.
pub fn save_index(entries: &Vec<IndexEntry>) -> io::Result<()> {
    let index_path = get_index_path();
    let file = File::create(&index_path)?;
    let mut writer = BufWriter::new(file);

    for entry in entries {
        write_index_entry(&mut writer, entry)?;
    }

    writer.flush()?;
    Ok(())
}

// This function will remove the file from the index file.
pub fn remove_from_index(file_path: &str) -> io::Result<()> {
    let mut entries = load_index()?;
    entries.retain(|entry| entry.path != file_path);
    save_index(&entries)?;
    Ok(())
}

// This function will update the index file with the new hash value.
pub fn update_index(file_path: &str, blob_hash: &str) -> io::Result<()> {
    let mut entries = load_index()?;
    for entry in &mut entries {
        if entry.path == file_path {
            entry.blob_hash = blob_hash.to_string();
            break;
        }
    }
    save_index(&entries)?;
    Ok(())
}

// This function will clear the index file.
pub fn clear_index() -> io::Result<()> {
    let index_path = get_index_path();
    // Check if the index file exists before attempting to clear it
    if index_path.exists() {
        File::create(&index_path)?; // Truncate the file if it exists
    }
    Ok(())
}

// This function will read an index entry from the index file.
fn read_index_entry<R: Read>(reader: &mut R) -> io::Result<IndexEntry> {
    use std::io::ErrorKind;

    // Read the mode
    let mut mode_bytes = [0u8; 4];
    if reader.read_exact(&mut mode_bytes).is_err() {
        return Err(io::Error::new(ErrorKind::UnexpectedEof, "No more entries"));
    }
    let mode = u32::from_be_bytes(mode_bytes);

    // Read the hash length
    let mut hash_len_bytes = [0u8; 1];
    reader.read_exact(&mut hash_len_bytes)?;
    let hash_len = u8::from_be_bytes(hash_len_bytes) as usize;

    // Read the hash
    let mut hash_bytes = vec![0u8; hash_len];
    reader.read_exact(&mut hash_bytes)?;
    let blob_hash = String::from_utf8(hash_bytes).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

    // Read the path length
    let mut path_len_bytes = [0u8; 2];
    reader.read_exact(&mut path_len_bytes)?;
    let path_len = u16::from_be_bytes(path_len_bytes) as usize;

    // Read the path
    let mut path_bytes = vec![0u8; path_len];
    reader.read_exact(&mut path_bytes)?;
    let path = String::from_utf8(path_bytes).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

    Ok(IndexEntry {
        mode,
        blob_hash,
        path
    })
}

// This function will write an index entry to the index file.
fn write_index_entry<W: Write>(writer: &mut W, entry: &IndexEntry) -> io::Result<()> {
    writer.write_all(&entry.mode.to_be_bytes())?;

    let hash_bytes = entry.blob_hash.as_bytes();
    let hash_len = hash_bytes.len() as u8;
    writer.write_all(&hash_len.to_be_bytes())?;
    writer.write_all(hash_bytes)?;

    let path_bytes = entry.path.as_bytes();
    let path_len = path_bytes.len() as u16;
    writer.write_all(&path_len.to_be_bytes())?;
    writer.write_all(path_bytes)?;

    Ok(())
}

fn create_index_from_path(directory: &Path) -> io::Result<Vec<IndexEntry>> {
    let mut index = Vec::new();

    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            // If the path is a directory, recurse into it
            if path.is_dir() {
                let subdir_state = create_index_from_path(&path).unwrap();
                index.extend(subdir_state);
            } else {
                // If it's a file, get its state
                let mut object = store_temporary(&path.to_string_lossy().to_string()).unwrap();

                let key = hash_data(&object)?;

                let entry = IndexEntry {
                    mode: 0o100644,
                    blob_hash: key.to_string(),
                    path: path.to_string_lossy().to_string(),
                };

                index.push(entry);    
                }
            }
        }
    Ok(index)
}

fn check_for_changes(previous_index_entry: &Vec<IndexEntry>, current_index_entry: &Vec<IndexEntry>) -> HashMap<String, String> {
    let mut changes = HashMap::new();

    // Create HashMap from path to blob_hash for quick lookup by borrowing values
    let previous_files: HashMap<String, String> = previous_index_entry.iter()
        .map(|entry| (entry.path.clone(), entry.blob_hash.clone()))
        .collect();

    let current_files: HashMap<String, String> = current_index_entry.iter()
        .map(|entry| (entry.path.clone(), entry.blob_hash.clone()))
        .collect();

    // Check for changes in current files
    for curr_index in current_index_entry.iter() {
        let curr_path = &curr_index.path;
        let curr_hash = &curr_index.blob_hash;

        match previous_files.get(curr_path) {
            Some(prev_hash) if prev_hash == curr_hash => {
                // File has not changed (same blob_hash and path)
                changes.insert(curr_path.clone(), "unmodified".to_string());
            }
            Some(_) => {
                // File content has changed (hash is different)
                changes.insert(curr_path.clone(), "modified".to_string());
            }
            None => {
                // New file detected (not present in previous index)
                changes.insert(curr_path.clone(), "new file".to_string());
            }
        }
    }

    // Check for removed files (present in previous index but not in current)
    for prev_index in previous_index_entry.iter() {
        let prev_path = &prev_index.path;
        if !current_files.contains_key(prev_path) {
            // File removed
            changes.insert(prev_path.clone(), "deleted".to_string());
        }
    }

    changes
}

pub fn file_changes(path: &Path) -> HashMap<String, String>{

    let previous_index_entry: Vec<IndexEntry> = load_index().unwrap();
    let current_index_entry = create_index_from_path(path).unwrap();

    let changes = check_for_changes(&previous_index_entry, &current_index_entry);

    changes
}

pub fn get_status(){
    let path = PathBuf::from(SOURCE_PATH);
    let result = file_changes(&path);

    let branch_name = branches::get_current_branch_name().unwrap();
    print!("On branch {}\n\n", branch_name);

    // Compares the tree with Index files 
    let tree_hash = get_current_tree_from_commit_hash();
    let tree_hashmap = convert_tree_entry_to_hashmap(tree_hash);
    let tree_index_entry = create_entry_from_hashmap(tree_hashmap);
    let current_index_entry = load_index().unwrap();
    let staged_changes = check_for_changes(&tree_index_entry, &current_index_entry);

    println!("Changes to be committed:\n    (use \"git reset HEAD <file>...\" to unstage)");
    for (path, change) in &staged_changes {
        println!("{}", format!("{}:   {}", change, path).green());
    }

    println!("\n\n");
    // compares the index files to current directory 
    println!("Changes not staged for commit:\n  (use \"rit add <file>... to update what will be committed)");
    for (path, change) in &result {
        if change == "modified" || change == "deleted" {
            println!("{}", format!("{}:   {}", change, path).red());
        }
    }


    println!("\n\n");
    // compares the index files to current directory 
    println!("Untracked files:\n    (use \"rit add <file>... to include in what will be committed)");
    for (path, change) in &result {
        if change == "new file"{
            println!("{}", format!("{}:   {}", change, path).red());
        }
    }


}

pub fn create_entry_from_hashmap(tree: HashMap<String, String>) -> Vec<IndexEntry>{
    tree.into_iter()
    .map(|(path, blob_hash)| IndexEntry {
        mode: 0o100644, 
        blob_hash,
        path,
    })
    .collect()
}

pub fn get_status_test(result: HashMap<String, String>,  staged_changes: HashMap<String, String>){

    println!("Changes to be committed:\n    (use \"git reset HEAD <file>...\" to unstage)");
    for (path, change) in &staged_changes {
        println!("{}", format!("{}:   {}", change, path).green());
    }

    println!("\n\n");
    // compares the index files to current directory 
    println!("Changes not staged for commit:\n  (use \"rit add <file>... to update what will be committed)");
    for (path, change) in &result {
        if change == "modified" || change == "deleted" {
            println!("{}", format!("{}:   {}", change, path).red());
        }
    }

    println!("\n\n");
    // compares the index files to current directory 
    println!("Untracked files:\n    (use \"rit add <file>... to include in what will be committed)");
    for (path, change) in &result {
        if change == "new file"{
            println!("{}", format!("{}:   {}", change, path).red());
        }
    }


}



#[cfg(test)]
mod tests {
    use crate::constants::SOURCE_PATH;

    use super::*;
    use std::fs;
    use std::io::Cursor;

    fn setup() {
        // Ensure the directory exists for testing
        let _ = fs::create_dir_all(DIRECTORY_PATH);
    }

    fn cleanup() {
        // Remove the index file after each test
        let _ = fs::remove_file(get_index_path());

        // Remove the directory after each test
        let _ = fs::remove_dir(DIRECTORY_PATH);
    }

    // TESTS
    #[test]
    fn test_create_index() {
        setup();
        // Ensure the index file doesn't exist before the test
        assert!(!get_index_path().exists());

        // Call create_index and check if the file is created
        create_index().unwrap();
        assert!(get_index_path().exists());

        cleanup();
    }

    #[test]
    fn test_add_to_index() {
        setup();
        create_index().unwrap(); // Ensure the index file exists

        // Add a file to the index
        add_to_index("test_file.txt", "hash123").unwrap();
        
        // Load the index and check the entry
        let entries = load_index().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "test_file.txt");
        assert_eq!(entries[0].blob_hash, "hash123");

        cleanup();
    }

    #[test]
    fn test_load_index_empty() {
        setup();
        create_index().unwrap();

        // Test loading an empty index
        let entries = load_index().unwrap();
        assert_eq!(entries.len(), 0);

        cleanup();
    }

    #[test]
    fn test_remove_from_index() {
        setup();
        create_index().unwrap();
        add_to_index("test_file.txt", "hash123").unwrap();
        add_to_index("test_file2.txt", "hash456").unwrap();

        // Remove a file from the index
        remove_from_index("test_file.txt").unwrap();

        // Load the index and check the remaining entries
        let entries = load_index().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "test_file2.txt");

        cleanup();
    }

    // Test Save Index
    #[test]
    fn test_save_index() {
        setup();
        create_index().unwrap();
        add_to_index("test_file.txt", "hash123").unwrap();
        add_to_index("test_file2.txt", "hash456").unwrap();

        // Save the index
        let entries = load_index().unwrap();
        save_index(&entries).unwrap();

        // Load the index and check the entries
        let new_entries = load_index().unwrap();
        assert_eq!(entries, new_entries);

        cleanup();
    }

    // Test write_index_entry
    #[test]
    fn test_write_index_entry() {
        // Arrange
        let entry = IndexEntry {
            mode: 0o100644, // Regular file mode
            blob_hash: "123abc".to_string(),
            path: "test_file.txt".to_string(),
        };

        // Create a buffer to write to (simulates a file in memory)
        let mut buffer: Vec<u8> = Vec::new();

        // Act
        write_index_entry(&mut buffer, &entry).unwrap();

        // Assert
        // Verify that the mode, hash, and path were written correctly
        // Expected bytes:
        // Mode: [00, 00, 81, A4] -> 0o100644
        // Hash length: [00, 06] (length of "123abc")
        // Hash: [31, 32, 33, 61, 62, 63] (ASCII for "123abc")
        // Path length: [00, 0d] (length of "test_file.txt")
        // Path: [74, 65, 73, 74, 5f, 66, 69, 6c, 65, 2e, 74, 78, 74] (ASCII for "test_file.txt")

        let expected_bytes: Vec<u8> = vec![
            0x00, 0x00, 0x81, 0xA4,  // Mode: 0o100644 -> u32 -> [0x00, 0x00, 0x81, 0xA4]
            0x06,              // Hash length: 6 (length of "123abc")
            0x31, 0x32, 0x33, 0x61, 0x62, 0x63, // Hash: "123abc"
            0x00, 0x0d,              // Path length: 13 (length of "test_file.txt")
            0x74, 0x65, 0x73, 0x74, 0x5f, 0x66, 0x69, 0x6c, 0x65, 0x2e, 0x74, 0x78, 0x74 // Path: "test_file.txt"
        ];

        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn test_read_index_entry() {
        // Arrange
        // Prepare the bytes as they would appear in the index file
        let entry_bytes: Vec<u8> = vec![
            0x00, 0x00, 0x81, 0xA4,        // Mode: 0o100644 -> u32 -> [0x00, 0x00, 0x81, 0xA4]
            0x06,                    // Hash length: 6
            0x31, 0x32, 0x33, 0x61, 0x62, 0x63, // Hash: "123abc"
            0x00, 0x0d,                    // Path length: 13 (length of "test_file.txt")
            0x74, 0x65, 0x73, 0x74, 0x5f, 0x66, 0x69, 0x6c, 0x65, 0x2e, 0x74, 0x78, 0x74 // Path: "test_file.txt"
        ];

        // Create a cursor (reader) over the byte data
        let mut reader = Cursor::new(entry_bytes);

        // Act
        let result = read_index_entry(&mut reader);

        // Assert
        assert!(result.is_ok());
        let entry = result.unwrap();

        // Check that the parsed data matches the expected IndexEntry
        assert_eq!(entry.mode, 0o100644); // Check file mode
        assert_eq!(entry.blob_hash, "123abc"); // Check hash
        assert_eq!(entry.path, "test_file.txt"); // Check file path
    }

    #[test]
    fn test_clear_index() {
        setup();
        // Arrange
        // Create the index file with some initial content
        create_index().unwrap();
        let index_path = get_index_path();
        let mut file = File::create(&index_path).unwrap();
        writeln!(file, "Some initial content").unwrap();

        // Act
        clear_index().unwrap();

        // Assert
        // Check that the file exists and is empty after calling clear_index
        let metadata = fs::metadata(&index_path).unwrap();
        assert!(metadata.is_file()); // The file should exist
        assert_eq!(metadata.len(), 0); // The file should be empty (size = 0)

        cleanup();
    }

    #[test]
    fn test_load_index() {
        setup();
        // Arrange
        create_index().unwrap();
        let index_path = get_index_path();

        // Write some entries to the index file directly in the format expected by the `load_index` function
        let mut file = File::create(&index_path).unwrap();
        let entry_bytes: Vec<u8> = vec![
            0x00, 0x00, 0x81, 0xA4,        // Mode: 0o100644 -> u32 -> [0x00, 0x00, 0x81, 0xA4]
            0x06,                    // Hash length: 6
            0x31, 0x32, 0x33, 0x61, 0x62, 0x63, // Hash: "123abc"
            0x00, 0x0d,                    // Path length: 13 (length of "test_file.txt")
            0x74, 0x65, 0x73, 0x74, 0x5f, 0x66, 0x69, 0x6c, 0x65, 0x2e, 0x74, 0x78, 0x74 // Path: "test_file.txt"
        ];

        file.write_all(&entry_bytes).unwrap();
        file.flush().unwrap();

        // Act
        let entries = load_index().unwrap();

        // Assert
        assert_eq!(entries.len(), 1); // We expect 1 entry to be loaded
        let entry = &entries[0];
        assert_eq!(entry.mode, 0o100644); // Check the mode
        assert_eq!(entry.blob_hash, "123abc"); // Check the hash
        assert_eq!(entry.path, "test_file.txt"); // Check the path

        // Clean up
        fs::remove_file(index_path).unwrap();

        cleanup();
    }

    #[test]
    fn test_update_index() {
        setup();
        // Arrange
        // Create the index file and add an entry
        create_index().unwrap();
        let index_path = get_index_path();

        // Create an initial entry in the index file
        let initial_entry_bytes: Vec<u8> = vec![
            0x00, 0x00, 0x81, 0xA4,        // Mode: 0o100644 -> u32 -> [0x00, 0x00, 0x81, 0xA4]
            0x06,                    // Hash length: 6
            0x31, 0x32, 0x33, 0x61, 0x62, 0x63, // Hash: "123abc"
            0x00, 0x0d,                    // Path length: 13 (length of "test_file.txt")
            0x74, 0x65, 0x73, 0x74, 0x5f, 0x66, 0x69, 0x6c, 0x65, 0x2e, 0x74, 0x78, 0x74 // Path: "test_file.txt"
        ];
        let mut file = File::create(&index_path).unwrap();
        file.write_all(&initial_entry_bytes).unwrap();
        file.flush().unwrap();

        // Act
        // Update the hash of the file in the index
        update_index("test_file.txt", "newhash").unwrap();

        // Assert
        // Reload the index and check if the hash has been updated
        let entries = load_index().unwrap();
        assert_eq!(entries.len(), 1); // There should still be 1 entry
        let entry = &entries[0];
        assert_eq!(entry.path, "test_file.txt"); // File path should remain the same
        assert_eq!(entry.blob_hash, "newhash"); // The hash should be updated to "newhash"

        // Clean up
        fs::remove_file(index_path).unwrap();

        cleanup();
    }

    // Helper function to create a test file
    fn create_test_file<P: AsRef<Path>>(path: P, content: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_create_index_from_path() {
        // Create a temporary directory to test in
        let temp_dir = TempDir::new("test_dir").unwrap();
        let temp_path = temp_dir.path();

        // Create some test files in the temporary directory
        let file1 = temp_path.join("file1.txt");
        create_test_file(&file1, "Hello, World!").unwrap();

        let file2 = temp_path.join("file2.txt");
        create_test_file(&file2, "Rust is awesome!").unwrap();

        // Create a subdirectory and add a file inside it
        let subdir = temp_path.join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file3 = subdir.join("file3.txt");
        create_test_file(&file3, "Subdirectory file content").unwrap();

        // Call create_index_from_path on the temporary directory
        let index = create_index_from_path(temp_path).unwrap();

        // Print out the index to see the results
        println!("Index: {:#?}", index);

        // Check that the index contains entries for the files
        assert_eq!(index.len(), 3);  // We expect 3 files (file1.txt, file2.txt, file3.txt)

        // Check if specific files exist in the index
        assert!(index.iter().any(|entry| entry.path == file1.to_string_lossy()));
        assert!(index.iter().any(|entry| entry.path == file2.to_string_lossy()));
        assert!(index.iter().any(|entry| entry.path == file3.to_string_lossy()));
    }


    // Function to print out the directory structure
    fn print_directory_structure<P: AsRef<Path>>(path: P, indent: usize) -> std::io::Result<()> {
        let path = path.as_ref();
        
        // Print the current directory or file with proper indentation
        println!("{:indent$}{}", "", path.display(), indent = indent);
        
        if path.is_dir() {
            // If it's a directory, print its contents recursively
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                print_directory_structure(entry_path, indent + 4)?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_create_index_from_path_multiple_subdirs() {
        // Create a temporary directory to test in
        let temp_dir = TempDir::new("test_dir").unwrap();
        let temp_path = temp_dir.path();

        // Create files in the root directory
        let file1 = temp_path.join("file1.txt");
        create_test_file(&file1, "Root file 1").unwrap();

        let file2 = temp_path.join("file2.txt");
        create_test_file(&file2, "Root file 2").unwrap();

        // Create a subdirectory and add files inside it
        let subdir1 = temp_path.join("subdir1");
        fs::create_dir(&subdir1).unwrap();
        let file3 = subdir1.join("file3.txt");
        create_test_file(&file3, "Subdir 1 file").unwrap();

        let subdir2 = temp_path.join("subdir2");
        fs::create_dir(&subdir2).unwrap();
        let file4 = subdir2.join("file4.txt");
        create_test_file(&file4, "Subdir 2 file").unwrap();

        // Create another level of subdirectories and files
        let subsubdir1 = subdir1.join("subsubdir1");
        fs::create_dir(&subsubdir1).unwrap();
        let file5 = subsubdir1.join("file5.txt");
        create_test_file(&file5, "Subsubdir 1 file").unwrap();

        let subsubdir2 = subdir2.join("subsubdir2");
        fs::create_dir(&subsubdir2).unwrap();
        let file6 = subsubdir2.join("file6.txt");
        create_test_file(&file6, "Subsubdir 2 file").unwrap();

        // Print the directory structure
        println!("\nDirectory Structure:");
        print_directory_structure(temp_path, 0).unwrap();

        // Call create_index_from_path on the temporary directory
        let index = create_index_from_path(temp_path).unwrap();

        // Print out the index to see the results
        println!("Index: {:#?}", index);

        // Check the expected number of entries (6 files in total)
        assert_eq!(index.len(), 6);

        // Check if specific files exist in the index
        assert!(index.iter().any(|entry| entry.path == file1.to_string_lossy()));
        assert!(index.iter().any(|entry| entry.path == file2.to_string_lossy()));
        assert!(index.iter().any(|entry| entry.path == file3.to_string_lossy()));
        assert!(index.iter().any(|entry| entry.path == file4.to_string_lossy()));
        assert!(index.iter().any(|entry| entry.path == file5.to_string_lossy()));
        assert!(index.iter().any(|entry| entry.path == file6.to_string_lossy()));
    }

    #[test]
    fn test_check_for_changes_no_file_changes() {
        // Define some sample index entries
        let previous_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "7ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(),
            },
        ];

        let current_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "7ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(),
            },
        ];
        
        let changes = check_for_changes(&previous_index_entry, &current_index_entry);

        let expected_changes = vec![
            ("/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(), "unmodified".to_string()),
            ("/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(), "unmodified".to_string()),
        ];

        // Assert that the changes match the expected values
        for (path, expected_change) in expected_changes {
            assert_eq!(changes.get(&path), Some(&expected_change));
        } 
    }

    #[test]
    fn test_check_for_changes_one_file_change() {
        // Define some sample index entries
        let previous_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "1ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(),
            },
        ];

        let current_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "7ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(),
            },
        ];

        let changes = check_for_changes(&previous_index_entry, &current_index_entry);

        let expected_changes = vec![
            ("/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(), "modified".to_string()),
            ("/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(), "unmodified".to_string()),
        ];

        // Assert that the changes match the expected values
        for (path, expected_change) in expected_changes {
            assert_eq!(changes.get(&path), Some(&expected_change));
        }    
    }

    #[test]
    fn test_check_for_changes_one_filename_change() {
        // Define some sample index entries
        let previous_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "1ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(),
            },
        ];

        let current_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "7ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file3.txt".to_string(),
            },
        ];

        let changes = check_for_changes(&previous_index_entry, &current_index_entry);
        
        // for (path, change) in &changes {
        //     println!("Path: {}, Change: {}", path, change);
        // }

        let expected_changes = vec![
            ("/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(), "modified".to_string()),
            ("/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(), "deleted".to_string()),
            ("/tmp/test_dir.z2hWBWkSguqs/file3.txt".to_string(), "new file".to_string()),
        ];

        // Assert that the changes match the expected values
        for (path, expected_change) in expected_changes {
            assert_eq!(changes.get(&path), Some(&expected_change));
        }    
    }

    #[test]
    fn test_check_for_changes_one_filename_change_v2() {
        // Define some sample index entries
        let previous_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "1ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file3.txt".to_string(),
            },
        ];

        let current_index_entry = vec![
            IndexEntry {
                mode: 33188,
                blob_hash: "7ac90a45302da0bd11bdb6d9ea02c4f9df215c5eec7c3a590436e850e9017fb".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(),
            },
            IndexEntry {
                mode: 33188,
                blob_hash: "b5278c6a1461eff7b70a2bb360e95f020e1303905dc26aa8d44b557a8ced1d12".to_string(),
                path: "/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(),
            },
        ];

        let changes = check_for_changes(&previous_index_entry, &current_index_entry);
        
        // for (path, change) in &changes {
        //     println!("Path: {}, Change: {}", path, change);
        // }

        let expected_changes = vec![
            ("/tmp/test_dir.z2hWBWkSguqs/file2.txt".to_string(), "new file".to_string()),
            ("/tmp/test_dir.z2hWBWkSguqs/file1.txt".to_string(), "modified".to_string()),
            ("/tmp/test_dir.z2hWBWkSguqs/file3.txt".to_string(), "deleted".to_string()),
        ];

        // Assert that the changes match the expected values
        for (path, expected_change) in expected_changes {
            assert_eq!(changes.get(&path), Some(&expected_change));
        }
            
    }

    #[test]
    fn test_file_changes() {
        let path = PathBuf::from(SOURCE_PATH);
        let result = file_changes(&path);
        // for (path, change) in &result {
        //     println!("Path: {}, Change: {}", path, change);
        // }
        assert!(!result.is_empty());  
    }

    #[test]
    fn test_get_status() {
        get_status();
    }

    #[test]
    fn test_get_status_test() {
        // result are from the current directory and the index entries
        let mut result = HashMap::new();
        result.insert("file1.rs".to_string(), "modified".to_string());
        result.insert("file2.rs".to_string(), "new file".to_string());
        result.insert("file3.rs".to_string(), "deleted".to_string());
    
        //staged entries are from the tree and index entries
        let mut staged_changes = HashMap::new();
        staged_changes.insert("file4.rs".to_string(), "modified".to_string());
        staged_changes.insert("file5.rs".to_string(), "added".to_string());
    
        // Call the function with the example data
        get_status_test(result, staged_changes);
    
    }

    #[test]
    fn test_get_status_two_staged_change() {
        // result are from the current directory and the index entries
        let mut result = HashMap::new();
        result.insert("file2.rs".to_string(), "new file".to_string());
    
        //staged entries are from the tree and index entries
        let mut staged_changes = HashMap::new();
        staged_changes.insert("file4.rs".to_string(), "modified".to_string());
        staged_changes.insert("file5.rs".to_string(), "added".to_string());
    
        // Call the function with the example data
        get_status_test(result, staged_changes);
    
    }
}
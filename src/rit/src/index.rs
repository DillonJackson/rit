// This file will contain the methods to interact with the index file.
// What is stored in the index file?
// The index file stores the file name, the hash value of the file, and the file path.


use crate::constants::{DIRECTORY_PATH, INDEX_FILE};
use std::fs::{File};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::{PathBuf, Path};

#[derive(Debug, Clone, PartialEq)]
pub struct IndexEntry {
    mode: u32,
    blob_hash: String,
    path: String
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
    let mut entries = load_index()?;

    // Check if the file is already in the index
    if entries.iter().any(|entry| entry.path == file_path){
        // Update the index entry
        update_index(file_path, blob_hash)?;
    } else {
        // Add a new entry to the index
        let entry = IndexEntry {
            mode: 0o100644, // Normal file
            blob_hash: blob_hash.to_string(),
            path: file_path.to_string()
        };
        entries.push(entry);
        save_index(&entries)?;
    }
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
    let mut hash_len_bytes = [0u8; 2];
    reader.read_exact(&mut hash_len_bytes)?;
    let hash_len = u16::from_be_bytes(hash_len_bytes) as usize;

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
    let hash_len = hash_bytes.len() as u16;
    writer.write_all(&hash_len.to_be_bytes())?;
    writer.write_all(hash_bytes)?;

    let path_bytes = entry.path.as_bytes();
    let path_len = path_bytes.len() as u16;
    writer.write_all(&path_len.to_be_bytes())?;
    writer.write_all(path_bytes)?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
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
            0x00, 0x06,              // Hash length: 6 (length of "123abc")
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
            0x00, 0x06,                    // Hash length: 6
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
            0x00, 0x06,                    // Hash length: 6
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
            0x00, 0x06,                    // Hash length: 6
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
}
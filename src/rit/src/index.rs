// This file will contain the methods to interact with the index file.
// What is stored in the index file?
// The index file stores the file name, the hash value of the file, and the file path.


use crate::constants::{DIRECTORY_PATH, INDEX_FILE};
use std::fs::{File};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::{PathBuf, Path};

#[derive(Debug, Clone)]
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
    File::create(&index_path)?;
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
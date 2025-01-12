// Git uses serilization for tree objects. We will use the same approach to serialize and deserialize tree objects.

use crate::database;
use crate::index::{IndexEntry};
use std::collections::{BTreeMap, HashMap};
use hex;
use json::iterators::Entries;
use std::io;
use std::path::{Path, PathBuf};
use crate::constants::{BLOB, TREE};

#[derive(Debug, Clone, PartialEq)]
pub struct TreeEntry {
    pub mode: u32,
    pub object_type: String, // "blob" or "tree"
    pub hash: String,
    pub name: String,
}


pub fn create_tree(index_entries: &[IndexEntry]) -> io::Result<String> {
    // Start recursive processing from the root directory
    let root_path: PathBuf = PathBuf::new();
    let tree_hash: String = recursive_tree(&root_path, index_entries)?;

    Ok(tree_hash)
}


fn recursive_tree(
    cur_dir: &PathBuf,
    entries: &[IndexEntry]
) -> io::Result<String> {
    let mut tree_entries: HashMap<String, TreeEntry> = HashMap::new();
    let mut sub_tree_entries: HashMap<String, Vec<IndexEntry>> = HashMap::new();

    // Filter entries that belong to the current directory
    for entry in entries {
        let entry_path = Path::new(&entry.path);
        let relative_path = entry_path.strip_prefix(&cur_dir).unwrap();

        let mut components = relative_path.components();

        if let Some(first_component) = components.next() {
            let name = first_component.as_os_str().to_str().unwrap().to_string();

            if components.next().is_none() {
                // Direct child of the current directory
                tree_entries.insert(
                    name.clone(),
                    TreeEntry {
                        mode: entry.mode,
                        object_type: "blob".to_string(),
                        hash: entry.blob_hash.clone(),
                        name,
                    },
                );
            } else {
                // Entry belongs to a subdirectory
                sub_tree_entries
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(entry.clone());
            }
        }
    }

    // Process subdirectories recursively
    for (sub_dir_name, sub_entries) in sub_tree_entries {
        let sub_dir_path = cur_dir.join(&sub_dir_name);
        let sub_tree_hash = recursive_tree(&sub_dir_path, &sub_entries)?;
        tree_entries.insert(
            sub_dir_name.clone(),
            TreeEntry {
                mode: 0o040000, // Default mode for a tree
                object_type: "tree".to_string(),
                hash: sub_tree_hash,
                name: sub_dir_name,
            },
        );
    }

    // Sort 
    let mut entries = tree_entries.values().cloned().collect::<Vec<TreeEntry>>();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    
    // Serialize the tree entries
    let serialized_tree = serialize_tree_entries(&entries)?;
    
    // Store the tree in the database and return its hash
    let hash = database::store_data(&serialized_tree, TREE)?;

    Ok(hash)
}

pub fn read_tree(tree_hash: &str) -> io::Result<Vec<TreeEntry>> {
    // Get the data for the tree object
    let (_, _, data) = database::get_data(tree_hash)?;

    // Deserialize the tree entries
    let entries = deserialize_tree_entries(&data)?;

    Ok(entries)
}

fn serialize_tree_entries(entries: &[TreeEntry]) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();

    for entry in entries {
        data.extend_from_slice(entry.mode.to_string().as_bytes());
        data.push(b' ');

        data.extend_from_slice(entry.name.as_bytes());
        data.push(0); // Null byte

        match hex::decode(&entry.hash){
            Ok(hash_byte) if hash_byte.len() == 32 => data.extend(hash_byte),
            Ok(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid sha256 hash length")),
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e))
        }
    }
    Ok(data)
}

fn deserialize_tree_entries(data: &[u8]) -> io::Result<Vec<TreeEntry>> {
    let mut entries: Vec<TreeEntry> = Vec::new();
    let mut i: usize = 0;

    while i < data.len() {
        // Read the mode
        let mut mode: Vec<u8> = Vec::new();
        while data[i] != b' ' {
            mode.push(data[i]);
            i += 1;
        }
        let mode_str = String::from_utf8(mode).expect("Invalid UTF-8");
        let mode = mode_str.parse::<u32>().expect("Invalid mode");
        i += 1; // Skip the space

        // Read the name
        let mut name = Vec::new();
        while data[i] != 0 {
            name.push(data[i]);
            i += 1;
        }
        let name = String::from_utf8(name).expect("Invalid UTF-8");
        i += 1; // Skip the null byte

        // Read the hash (32 bytes for SHA-256)
        let hash_bytes = &data[i..i + 32];
        let hash = hex::encode(hash_bytes);
        i += 32;

        // Determine the object type based on the mode
        let object_type = if mode == 0o040000 {
            "tree".to_string()
        } else {
            "blob".to_string()
        };

        let entry = TreeEntry {
            mode,
            object_type,
            hash,
            name,
        };
        entries.push(entry);
    }

    Ok(entries)
}

pub fn convert_tree_entry_to_hashmap(entries: Vec<TreeEntry>) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for entry in entries {
        if entry.object_type == "blob" {
            result.insert(entry.name, entry.hash);
        }
    }

    result
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use std::io;

    #[test]
    fn test_serialize_deserialize_tree_entries() -> io::Result<()> {
        let entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: "06bf71aad1d68e12dca830259bf0bea1cd724468fb44e02a6b4fe425fcb11bb0".to_string(),
                name: "file1.txt".to_string(),
            },
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "c42edefc75871e4ce2146fcda67d03dda05cc26fdf93b17b55f42c1eadfdc322".to_string(),
                name: "dir".to_string(),
            },
        ];

        let serialized = serialize_tree_entries(&entries)?;
        print!("{:?}", serialized);
        let deserialized = deserialize_tree_entries(&serialized)?;

        assert_eq!(entries, deserialized);

        Ok(())
    }

    #[test]
    fn test_write_and_read_tree() -> io::Result<()> {
        // Prepare mock index entries
        let file1_hash = database::store_data(b"content of file1.txt", BLOB)?;
        let file2_hash = database::store_data(b"content of file2.txt", BLOB)?;
        let file3_hash = database::store_data(b"content of file3.txt", BLOB)?;

        let file1_up_hash = database::store_data(b"updated content of file1.txt", BLOB)?;
        let file2_up_hash = database::store_data(b"updated content of file2.txt", BLOB)?;
        let file4_hash = database::store_data(b"content of file4.txt", BLOB)?;
        let file5_hash = database::store_data(b"content of file5.txt", BLOB)?;

        let index_entries = vec![
            IndexEntry {
                mode: 0o100644,
                blob_hash: file1_hash.clone(),
                path: "file1.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: file2_hash.clone(),
                path: "dir/file2.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: file3_hash.clone(),
                path: "dir/subdir/file3.txt".to_string(),
            },
        ];

        // Write the tree
        let tree_hash = create_tree(&index_entries)?;
            print!("{:?}", tree_hash);
        // Read the tree
        let root_entries = read_tree(&tree_hash)?;

        // Check root entries
        assert_eq!(root_entries.len(), 2);

        let expected_root_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "70db42a48939d1d7ccbe5181c368841fae023a08857f48cadf62b1322b8643ee".to_string(), // We'll fill this in
                name: "dir".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: file1_hash.clone(),
                name: "file1.txt".to_string(),
            },
        ];

        // check if the tree objects is the same
        let tree_entry = expected_root_entries.iter().find(|e| e.name == "dir").unwrap();
        let tree_entry2 = root_entries.iter().find(|e| e.name == "dir").unwrap();
        assert_eq!(tree_entry, tree_entry2);

        // check if the blob objects is the same
        let blob_entry = expected_root_entries.iter().find(|e| e.name == "file1.txt").unwrap();
        let blob_entry2 = root_entries.iter().find(|e| e.name == "file1.txt").unwrap();
        assert_eq!(blob_entry, blob_entry2);
        
        // Read the 'dir' tree
        let dir_entries = read_tree(&tree_entry2.hash)?;

        // Check 'dir' entries
        assert_eq!(dir_entries.len(), 2);

        let expected_dir_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "f84b4f567ada0bd4c4c70b36389b9114b0b630f845bbc789235a012f64955c7b".to_string(), // We'll fill this in
                name: "subdir".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: index_entries[1].blob_hash.clone(),
                name: "file2.txt".to_string(),
            },
        ];

        // check if the tree objects is the same
        let tree_entry = expected_dir_entries.iter().find(|e| e.name == "subdir").unwrap();
        let tree_entry2 = dir_entries.iter().find(|e| e.name == "subdir").unwrap();
        assert_eq!(tree_entry, tree_entry2);

        // check if the blob objects is the same
        let blob_entry = expected_dir_entries.iter().find(|e| e.name == "file2.txt").unwrap();
        let blob_entry2 = dir_entries.iter().find(|e| e.name == "file2.txt").unwrap();
        assert_eq!(blob_entry, blob_entry2);

        // Read the 'subdir' tree
        let subdir_entries = read_tree(&tree_entry2.hash)?;

        // Check 'subdir' entries
        assert_eq!(subdir_entries.len(), 1);

        let expected_subdir_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: file3_hash.clone(),
                name: "file3.txt".to_string(),
            },
        ];

        assert_eq!(subdir_entries, expected_subdir_entries);

        // Simulate changes: update file1.txt, add file4.txt, and update file3.txt
        let updated_index_entries = vec![
            IndexEntry {
                mode: 0o100644,
                blob_hash: file1_up_hash.clone(),
                path: "file1.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: file2_up_hash.clone(),
                path: "dir/file2.txt".to_string(),
            },
            // Leave file3.txt unchanged so we can reuse the hash
            IndexEntry {
                mode: 0o100644,
                blob_hash: file3_hash.clone(),
                path: "dir/subdir/file3.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: file4_hash.clone(),
                path: "dir/file4.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: file5_hash.clone(),
                path: "dir2/subdir2/file5.txt".to_string(),
            },
        ];

        // Write the updated tree
        let updated_tree_hash = create_tree(&updated_index_entries)?;
        println!("Updated tree hash: {:?}", updated_tree_hash);

        // Read the updated tree
        let updated_root_entries = read_tree(&updated_tree_hash)?;

        // Check updated root entries
        assert_eq!(updated_root_entries.len(), 3);

        let expected_updated_root_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "3dd70afdee4ffc1dc741c162effe3eb142e8b0eb68e1f6dee0759bf8679381e3".to_string(), // We'll fill this in
                name: "dir".to_string(),
            },
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "b9f70f7df0d2064e9e860e74323dedc1a6ef3c800270f5492a7a2d8383c627f4".to_string(), // We'll fill this in
                name: "dir2".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: file1_up_hash.clone(),
                name: "file1.txt".to_string(),
            },
        ];
        
        // check if the tree objects is the same
        let tree_entry = updated_root_entries.iter().find(|e| e.name == "dir").unwrap();
        let tree_entry2 = expected_updated_root_entries.iter().find(|e| e.name == "dir").unwrap();
        assert_eq!(tree_entry, tree_entry2);
        
        // check if the tree objects is the same
        let tree_entry_dir_2 = updated_root_entries.iter().find(|e| e.name == "dir2").unwrap();
        let tree_entry2_dir_2 = expected_updated_root_entries.iter().find(|e| e.name == "dir2").unwrap();
        assert_eq!(tree_entry_dir_2, tree_entry2_dir_2);
        
        // check if the blob objects is the same
        let blob_entry = updated_root_entries.iter().find(|e| e.name == "file1.txt").unwrap();
        let blob_entry2 = expected_updated_root_entries.iter().find(|e| e.name == "file1.txt").unwrap();
        assert_eq!(blob_entry, blob_entry2);
        

        // Read the updated 'dir' tree
        let updated_dir_entries = read_tree(&tree_entry.hash)?;

        // Check updated 'dir' entries
        assert_eq!(updated_dir_entries.len(), 3);

        let expected_updated_dir_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "f84b4f567ada0bd4c4c70b36389b9114b0b630f845bbc789235a012f64955c7b".to_string(), // We'll fill this in
                name: "subdir".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: file2_up_hash.clone(),
                name: "file2.txt".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: file4_hash.clone(),
                name: "file4.txt".to_string(),
            },
        ];

        // check if the tree objects is the same
        let tree_entry = updated_dir_entries.iter().find(|e| e.name == "subdir").unwrap();
        let tree_entry2 = expected_updated_dir_entries.iter().find(|e| e.name == "subdir").unwrap();
        assert_eq!(tree_entry, tree_entry2);
        
        // check if the blob objects is the same
        let blob_entry = updated_dir_entries.iter().find(|e| e.name == "file2.txt").unwrap();
        let blob_entry2 = expected_updated_dir_entries.iter().find(|e| e.name == "file2.txt").unwrap();
        assert_eq!(blob_entry, blob_entry2);
        
        // check if the blob objects is the same
        let blob_entry = updated_dir_entries.iter().find(|e| e.name == "file4.txt").unwrap();
        let blob_entry2 = expected_updated_dir_entries.iter().find(|e| e.name == "file4.txt").unwrap();
        assert_eq!(blob_entry, blob_entry2);
        
        // Read the updated 'subdir' tree
        let updated_subdir_entries = read_tree(&tree_entry.hash)?;

        // Check updated 'subdir' entries
        assert_eq!(updated_subdir_entries.len(), 1);

        let expected_updated_subdir_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: updated_index_entries[2].blob_hash.clone(),
                name: "file3.txt".to_string(),
            },
        ];

        assert_eq!(updated_subdir_entries, expected_updated_subdir_entries);

        // Read the updated 'dir2' tree
        let updated_dir2_entries = read_tree(&tree_entry_dir_2.hash)?;

        // Check updated 'dir2' entries
        assert_eq!(updated_dir2_entries.len(), 1);

        let expected_updated_dir2_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "f974aef11d8693240620875b111bc7294abbb9ef42dfecd7a065c7997f616150".to_string(), // We'll fill this in
                name: "subdir2".to_string(),
            },
        ];

        assert_eq!(updated_dir2_entries, expected_updated_dir2_entries);
        
        // Read the updated 'subdir2' tree
        let updated_subdir2_tree_hash = &updated_dir2_entries.iter().find(|e| e.name == "subdir2").unwrap().hash;
        let updated_subdir2_entries = read_tree(updated_subdir2_tree_hash)?;

        // Check updated 'subdir2' entries
        assert_eq!(updated_subdir2_entries.len(), 1);

        let expected_updated_subdir2_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: file5_hash.clone(),
                name: "file5.txt".to_string(),
            },
        ];

        assert_eq!(updated_subdir2_entries, expected_updated_subdir2_entries);

        Ok(())
    }
}
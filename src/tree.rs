// Git uses serilization for tree objects. We will use the same approach to serialize and deserialize tree objects.

use crate::database;
use crate::index::{IndexEntry};
use std::collections::{BTreeMap, HashMap};
use hex;
use json::iterators::Entries;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct TreeEntry {
    pub mode: u32,
    pub object_type: String, // "blob" or "tree"
    pub hash: String,
    pub name: String,
}


pub fn create_tree(index_entries: &[IndexEntry], base_tree_hash: Option<&str>) -> io::Result<String> {

    // Convert index entries to a map for easier processing
    let mut path_to_entry: HashMap<String, IndexEntry> = HashMap::new();
    for entry in index_entries {
        path_to_entry.insert(entry.path.clone(), entry.clone());
    }

    // Start recursive processing from the root directory
    let root_path: PathBuf = PathBuf::new();
    let tree_hash: String = recursive_tree(root_path, base_tree_hash, index_entries.to_vec())?;

    Ok(tree_hash)
}


fn recursive_tree(
    cur_dir: PathBuf,
    cur_tree_hash: Option<&str>,
    entries: Vec<IndexEntry>,
) -> io::Result<String> {
    let mut tree_entries = BTreeMap::new();

    // Get entries for the current directory
    let mut cur_dir_blobs = Vec::new();
    let mut cur_dir_subdirs = HashMap::new();

    for entry in entries.iter() {
        let path = Path::new(&entry.path);
        if let Ok(relative_path) = path.strip_prefix(&cur_dir) {
            let mut components = relative_path.components();
            if let Some(comp) = components.next() {
                if components.clone().count() == 0 {
                    // This is a blob in the current directory
                    cur_dir_blobs.push(entry.clone());
                } else {
                    // This is part of a subdirectory
                    let dir_name = comp.as_os_str().to_str().unwrap().to_string();
                    cur_dir_subdirs
                        .entry(dir_name)
                        .or_insert_with(Vec::new)
                        .push(entry.clone());
                }
            }
        }
    }

    let existing_entries = if let Some(hash) = cur_tree_hash {
        read_tree(hash)?
    } else {
        Vec::new()
    };

    // Include existing blob entries if they're unchanged
    for existing_entry in &existing_entries {
        if existing_entry.object_type == "blob"
            && !cur_dir_blobs.iter().any(|e| {
                let e_last_component = Path::new(&e.path).components().last().unwrap();
                let existing_last_component = Path::new(&existing_entry.name).components().last().unwrap();
                e_last_component == existing_last_component
            })
        {
            tree_entries.insert(
                existing_entry.name.clone(),
                existing_entry.clone(),
            );
        }
    }

    // Add or update blob entries in the current directory
    for blob in cur_dir_blobs {
        let name = Path::new(&blob.path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        tree_entries.insert(
            name.clone(),
            TreeEntry {
                mode: blob.mode,
                object_type: "blob".to_string(),
                hash: blob.blob_hash,
                name,
            },
        );
    }

    // Recursively handle subdirectories
    for (subdir_name, subdir_entries) in cur_dir_subdirs {
        let subdir_path = cur_dir.join(&subdir_name);

        // Find the existing hash for this subdirectory in the current tree (if it exists)
        let subdir_hash = get_subtree_hash_from_base(&existing_entries, &subdir_name).ok();

        // Recurse into the next directory
        let subtree_hash = recursive_tree(subdir_path, subdir_hash.as_deref(), subdir_entries)?;

        // Add the tree entry for this subdirectory
        tree_entries.insert(
            subdir_name.clone(),
            TreeEntry {
                mode: 0o040000, // Directory mode
                object_type: "tree".to_string(),
                hash: subtree_hash,
                name: subdir_name,
            },
        );
    }

    // Serialize and store the current directory's tree
    let serialized_data = serialize_tree_entries(&tree_entries.values().cloned().collect::<Vec<_>>())?;
    let tree_hash = database::store_data(&serialized_data)?;

    Ok(tree_hash)
}

fn get_subtree_hash_from_base(base_entries: &Vec<TreeEntry>, sub_name: &str) -> io::Result<String> {
    // Find the subtree entry with the matching name and return its hash
    if let Some(entry) = base_entries.iter().find(|e| e.name == sub_name && e.object_type == "tree") {
        Ok(entry.hash.clone())
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Subtree not found"))
    }
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

pub fn read_tree(tree_hash: &str) -> io::Result<Vec<TreeEntry>> {
    // Get the data for the tree object
    let data = database::get_data(tree_hash)?;

    // Deserialize the tree entries
    let entries = deserialize_tree_entries(&data)?;

    Ok(entries)
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
        let index_entries = vec![
            IndexEntry {
                mode: 0o100644,
                blob_hash: database::store_data(b"content of file1.txt")?,
                path: "file1.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: database::store_data(b"content of file2.txt")?,
                path: "dir/file2.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: database::store_data(b"content of file3.txt")?,
                path: "dir/subdir/file3.txt".to_string(),
            },
        ];

        // Write the tree
        let tree_hash = create_tree(&index_entries, None)?;
            print!("{:?}", tree_hash);
        // Read the tree
        let root_entries = read_tree(&tree_hash)?;

        // Check root entries
        assert_eq!(root_entries.len(), 2);

        let mut expected_root_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "dir".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: index_entries[0].blob_hash.clone(),
                name: "file1.txt".to_string(),
            },
        ];

        // Get the hash of the 'dir' tree
        let dir_tree_hash = &root_entries.iter().find(|e| e.name == "dir").unwrap().hash;
            print!("dir_tree_hash: {:?}", dir_tree_hash);
        expected_root_entries[0].hash = dir_tree_hash.clone();

        assert_eq!(root_entries, expected_root_entries);

        // Read the 'dir' tree
        let dir_entries = read_tree(dir_tree_hash)?;

        // Check 'dir' entries
        assert_eq!(dir_entries.len(), 2);

        let mut expected_dir_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "subdir".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: index_entries[1].blob_hash.clone(),
                name: "file2.txt".to_string(),
            },
        ];

        // Get the hash of the 'subdir' tree
        let subdir_tree_hash = &dir_entries.iter().find(|e| e.name == "subdir").unwrap().hash;

        expected_dir_entries[0].hash = subdir_tree_hash.clone();

        // check if the tree objects is the same
        let tree_entry = expected_dir_entries.iter().find(|e| e.name == "subdir").unwrap();
        let tree_entry2 = dir_entries.iter().find(|e| e.name == "subdir").unwrap();
        assert_eq!(tree_entry, tree_entry2);

        // check if the blob objects is the same
        let blob_entry = expected_dir_entries.iter().find(|e| e.name == "file2.txt").unwrap();
        let blob_entry2 = dir_entries.iter().find(|e| e.name == "file2.txt").unwrap();
        assert_eq!(blob_entry, blob_entry2);

        // Read the 'subdir' tree
        let subdir_entries = read_tree(subdir_tree_hash)?;

        // Check 'subdir' entries
        assert_eq!(subdir_entries.len(), 1);

        let expected_subdir_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: index_entries[2].blob_hash.clone(),
                name: "file3.txt".to_string(),
            },
        ];

        assert_eq!(subdir_entries, expected_subdir_entries);

        // Simulate changes: update file1.txt, add file4.txt, and update file3.txt
        let updated_index_entries = vec![
            IndexEntry {
                mode: 0o100644,
                blob_hash: database::store_data(b"updated content of file1.txt")?,
                path: "file1.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: database::store_data(b"updated content of file2.txt")?,
                path: "dir/file2.txt".to_string(),
            },
            // Leave file3.txt unchanged so we can reuse the hash
            IndexEntry {
                mode: 0o100644,
                blob_hash: database::store_data(b"content of file4.txt")?,
                path: "dir/file4.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: database::store_data(b"content of file5.txt")?,
                path: "dir2/subdir2/file5.txt".to_string(),
            },
        ];

        // Write the updated tree
        let updated_tree_hash = create_tree(&updated_index_entries, Some(&tree_hash))?;
        println!("Updated tree hash: {:?}", updated_tree_hash);

        // Read the updated tree
        let updated_root_entries = read_tree(&updated_tree_hash)?;

        // Check updated root entries
        assert_eq!(updated_root_entries.len(), 3);

        let mut expected_updated_root_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "dir".to_string(),
            },
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "dir2".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: updated_index_entries[0].blob_hash.clone(),
                name: "file1.txt".to_string(),
            },
        ];

        // Get the hash of the updated 'dir' tree
        let updated_dir_tree_hash = &updated_root_entries.iter().find(|e| e.name == "dir").unwrap().hash;
        println!("updated_dir_tree_hash: {:?}", updated_dir_tree_hash);
        expected_updated_root_entries[0].hash = updated_dir_tree_hash.clone();

        // Get the hash of the updated 'dir2' tree
        let updated_dir2_tree_hash = &updated_root_entries.iter().find(|e| e.name == "dir2").unwrap().hash;
        println!("updated_dir2_tree_hash: {:?}", updated_dir2_tree_hash);
        expected_updated_root_entries[1].hash = updated_dir2_tree_hash.clone();

        assert_eq!(updated_root_entries, expected_updated_root_entries);

        // Read the updated 'dir' tree
        let updated_dir_entries = read_tree(updated_dir_tree_hash)?;

        // Check updated 'dir' entries
        assert_eq!(updated_dir_entries.len(), 3);

        let mut expected_updated_dir_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "subdir".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: updated_index_entries[1].blob_hash.clone(),
                name: "file2.txt".to_string(),
            },
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: updated_index_entries[2].blob_hash.clone(),
                name: "file4.txt".to_string(),
            },
        ];

        // Get the hash of the updated 'subdir' tree
        let updated_subdir_tree_hash = &updated_dir_entries.iter().find(|e| e.name == "subdir").unwrap().hash;

        expected_updated_dir_entries[0].hash = updated_subdir_tree_hash.clone();

        assert_eq!(updated_dir_entries, expected_updated_dir_entries);

        // Read the updated 'subdir' tree
        let updated_subdir_entries = read_tree(updated_subdir_tree_hash)?;

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
        let updated_dir2_entries = read_tree(updated_dir2_tree_hash)?;

        // Check updated 'dir2' entries
        assert_eq!(updated_dir2_entries.len(), 1);

        let mut expected_updated_dir2_entries = vec![
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "subdir2".to_string(),
            },
        ];

        // Get the hash of the updated 'subdir2' tree
        let updated_subdir2_tree_hash = &updated_dir2_entries.iter().find(|e| e.name == "subdir2").unwrap().hash;

        expected_updated_dir2_entries[0].hash = updated_subdir2_tree_hash.clone();

        assert_eq!(updated_dir2_entries, expected_updated_dir2_entries);

        // Read the updated 'subdir2' tree
        let updated_subdir2_entries = read_tree(updated_subdir2_tree_hash)?;

        // Check updated 'subdir2' entries
        assert_eq!(updated_subdir2_entries.len(), 1);

        let expected_updated_subdir2_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: updated_index_entries[4].blob_hash.clone(),
                name: "file5.txt".to_string(),
            },
        ];

        assert_eq!(updated_subdir2_entries, expected_updated_subdir2_entries);

        Ok(())
    }
}
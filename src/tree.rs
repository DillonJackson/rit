// Git uses serilization for tree objects. We will use the same approach to serialize and deserialize tree objects.

use crate::database;
use crate::index::{self, IndexEntry};
use std::collections::{BTreeMap, HashMap};
use std::ops::Index;
use hex;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct TreeEntry {
    pub mode: u32,
    pub object_type: String, // "blob" or "tree"
    pub hash: String,
    pub name: String,
}


pub fn write_tree(index_entries: &[IndexEntry]) -> io::Result<String> {

    // Convert index entries to a map for easier processing
    let mut path_to_entry: HashMap<String, IndexEntry> = HashMap::new();
    for entry in index_entries {
        path_to_entry.insert(entry.path.clone(), entry.clone());
    }

    // Start recursive processing from the root directory
    let root_path: PathBuf = PathBuf::new();
    let tree_hash: String = write_tree_recursive(&root_path, &path_to_entry)?;

    Ok(tree_hash)
}

fn write_tree_recursive(
    dir_path: &Path,
    path_to_entry: &HashMap<String, IndexEntry>,
) -> io::Result<String> {
    let mut entries = BTreeMap::new();

    // Separate paths into files in the current directory and subdirectories
    for (path_str, index_entry) in path_to_entry {
        let path = Path::new(path_str);
        if let Ok(relative_path) = path.strip_prefix(dir_path) {
            let mut components = relative_path.components();

            match components.next() {
                Some(comp) if components.clone().count() == 0 => {
                    // Current directory file
                    let name = comp.as_os_str().to_str().unwrap().to_string();
                    entries.insert(
                        name.clone(),
                        TreeEntry {
                            mode: index_entry.mode,
                            object_type: "blob".to_string(),
                            hash: index_entry.blob_hash.clone(),
                            name,
                        },
                    );
                }
                Some(comp) => {
                    // Subdirectory
                    let dir_name = comp.as_os_str().to_str().unwrap().to_string();
                    entries.entry(dir_name.clone()).or_insert_with(|| TreeEntry {
                        mode: 0o040000,
                        object_type: "tree".to_string(),
                        hash: String::new(), // Placeholder hash to be set after recursion
                        name: dir_name.clone(),
                    });
                }
                None => continue,
            }
        }
    }

    // Recursively process each subdirectory and update its hash
    for (dir_name, entry) in entries.iter_mut().filter(|(_, e)| e.object_type == "tree") {
        let subdir_path = dir_path.join(dir_name);
        entry.hash = write_tree_recursive(&subdir_path, path_to_entry)?;
    }

    // Serialize and store the tree entries
    let serialized_data = serialize_tree_entries(&entries.values().cloned().collect::<Vec<_>>())?;
    let tree_hash = database::store_data(&serialized_data)?;

    Ok(tree_hash)
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
    use crate::index::IndexEntry;
    use std::collections::HashMap;
    use std::io::{self, Error, ErrorKind};

    // // Mock database module
    // mod mock_database {
    //     use std::cell::RefCell;
    //     use std::collections::HashMap;
    //     use std::io;

    //     thread_local! {
    //         static STORE: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
    //     }

    //     pub fn store_data(data: &[u8]) -> io::Result<String> {
    //         let key = crate::hash::hash_data(data)?;
    //         STORE.with(|store| {
    //             store.borrow_mut().insert(key.clone(), data.to_vec());
    //         });
    //         Ok(key)
    //     }

    //     pub fn get_data(key: &str) -> io::Result<Vec<u8>> {
    //         STORE.with(|store| {
    //             if let Some(data) = store.borrow().get(key) {
    //                 Ok(data.clone())
    //             } else {
    //                 Err(io::Error::new(io::ErrorKind::NotFound, "Object not found"))
    //             }
    //         })
    //     }
    // }

    // // Mock hash module
    // mod mock_hash {
    //     use std::io;

    //     use md5;

    //     pub fn hash_data(data: &[u8]) -> io::Result<String> {
    //         // For testing purposes, we'll just return a fixed hash
    //         Ok(format!("{:040x}", md5::compute(data)))
    //     }
    // }

    // // Use the mock modules in tests
    // #[allow(unused_imports)]
    // use self::mock_database as database;
    // #[allow(unused_imports)]
    // use self::mock_hash as hash;

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
                blob_hash: "c42edefc75871e4ce2146fcda67d03dda05cc26fdf93b17b55f42c1eadfdc322".to_string(),
                path: "file1.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: "4bac27393bdd9777ce02453256c5577cd02275510b2227f473d03f533924f877".to_string(),
                path: "dir/file2.txt".to_string(),
            },
            IndexEntry {
                mode: 0o100644,
                blob_hash: "4bac27393bdd9777ce02453256c5577cd02275510b2227f473d03f533924f877".to_string(),
                path: "dir/subdir/file3.txt".to_string(),
            },
        ];

        // Write the tree
        let tree_hash = write_tree(&index_entries)?;

        // Read the tree
        let root_entries = read_tree(&tree_hash)?;

        // Check root entries
        assert_eq!(root_entries.len(), 2);

        let mut expected_root_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: "a1b2c3d4e5f6g7h8i9j0".to_string(),
                name: "file1.txt".to_string(),
            },
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "dir".to_string(),
            },
        ];

        // Get the hash of the 'dir' tree
        let dir_tree_hash = &root_entries.iter().find(|e| e.name == "dir").unwrap().hash;

        expected_root_entries[1].hash = dir_tree_hash.clone();

        assert_eq!(root_entries, expected_root_entries);

        // Read the 'dir' tree
        let dir_entries = read_tree(dir_tree_hash)?;

        // Check 'dir' entries
        assert_eq!(dir_entries.len(), 2);

        let mut expected_dir_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: "1234567890abcdef1234".to_string(),
                name: "file2.txt".to_string(),
            },
            TreeEntry {
                mode: 0o040000,
                object_type: "tree".to_string(),
                hash: "".to_string(), // We'll fill this in
                name: "subdir".to_string(),
            },
        ];

        // Get the hash of the 'subdir' tree
        let subdir_tree_hash = &dir_entries.iter().find(|e| e.name == "subdir").unwrap().hash;

        expected_dir_entries[1].hash = subdir_tree_hash.clone();

        assert_eq!(dir_entries, expected_dir_entries);

        // Read the 'subdir' tree
        let subdir_entries = read_tree(subdir_tree_hash)?;

        // Check 'subdir' entries
        assert_eq!(subdir_entries.len(), 1);

        let expected_subdir_entries = vec![
            TreeEntry {
                mode: 0o100644,
                object_type: "blob".to_string(),
                hash: "abcdef1234567890abcd".to_string(),
                name: "file3.txt".to_string(),
            },
        ];

        assert_eq!(subdir_entries, expected_subdir_entries);

        Ok(())
    }
}
use sha2::{Sha256, Digest};
use std::io::{self, BufReader, Cursor, BufRead};
use crate::utility::{create_directory,create_file, open_file};
use crate::compression::{uncompress_data};
use json::{JsonValue,parse};
use std::path::Path;
use std::fs;
use std::path::PathBuf;


//test folder
const DIRECTORY_PATH: &str = ".rit/files";

// create tree
pub fn create_tree()-> io::Result<()>{
    let path: &Path = Path::new(&DIRECTORY_PATH);
    let key = match tree_init(path) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error {}", e);
            return Err(e.into());
        }
    };
    println!(" root tree SHA-256 hash: {}", key);
    Ok(())
}

// Function to create a new file object
fn create_json_obj(file_type: &str, hash_value: &str, filename: &str) -> JsonValue {
    let mut file = JsonValue::new_object();
    file["type"] = file_type.into();
    file["hashvalue"] = hash_value.into();
    file["filename"] = filename.into();
    file
}

fn tree_init(dir: &Path)-> io::Result<String> {
    // Create a vector to hold multiple file objects
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a directory or a file
        if path.is_dir() {
            // Get the directory hash
            let dir_hash = tree_init(&path)?;

            //Get the directory name
            let dir_name = {
                if let Some(name) = path.file_name() {
                    name.to_string_lossy().to_string() // Convert and return the name
                } else {
                    String::new() // Default value if None
                }
            };
            
            //stores the directory entry
            let json_obj = create_json_obj("tree", &dir_hash, &dir_name);

            let json_string = json_obj.dump();

            // Convert the String to a Vec<u8>
            let json_bytes: Vec<u8> = json_string.into_bytes();
            files.extend(json_bytes);
            files.push(b'\n'); // Add a newline after each entr
        } else if path.is_file() {

            let file_hash = store_file(&path.display().to_string())?;
    
            // Get the file name
            let file_name = {
                if let Some(name) = path.file_name() {
                    name.to_string_lossy().to_string() // Convert and return the name
                } else {
                    String::new() // Default value if None
                }
            };

            // Creates a json obj
            let json_obj = create_json_obj("blob", &file_hash, &file_name);

            let json_string = json_obj.dump();

            // Must conver the String to a Vec<u8> 
            let json_bytes: Vec<u8> = json_string.into_bytes();

            // Stores data as a 1D vector
            files.extend(json_bytes);
            files.push(b'\n'); // Add a newline after each entr
        }
    }
    
    //hashing the vector
    let key = match store_buffer(&files) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error storing buffer: {}", e);
            return Err(e);
        },
    };

    // println!(" tree SHA-256 hash: {}", key);
    Ok(key)
}

// hash the file, then returns the key of the file
fn hash_file(buffer: &Vec<u8>) -> io::Result<String> {
    // Create a new SHA-256 hasher
    let mut hasher = Sha256::new();
    
    // Update the hasher with the file contents
    hasher.update(&buffer);
    
    // Finalize the hash and convert it to a hexadecimal string
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}


//storing a file
pub fn store_file(file_path: &str) -> io::Result<String> {
    // Open the file in read-only mode
    let path = PathBuf::from(file_path);
    let buffer = open_file(&path)?;
    //hash the file to obtain the key
    let key = match store_buffer(&buffer) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error storing buffer: {}", e);
            return Err(e);
        },
    };
    Ok(key)
}

//storing a Vec<u8>
fn store_buffer(buffer: &Vec<u8>) -> io::Result<String>{

    let key = match hash_file(&buffer) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error calculating hash: {}", e);
            return Err(e);
        },
    };
    
    // Print the stored hash value
    // println!("SHA-256 hash: {}", key);

    //directory filename
    let sub_dir_name: String = key.chars().take(2).collect();
    // println!("SHA-256 hash first 2 char: {}", sub_dir_name);

    //file filename
    let filename: String = key.chars().skip(2).collect();
    // println!("SHA-256 hash first 2 char: {}", filename);


    //create directory for the objects
    create_directory(".rit/objects", &sub_dir_name)?;

    //creates the file
    let result: String = format!("{}/{}", ".rit/objects", sub_dir_name);
    let result_str: &str = &result;
    // println!("dir {}", result_str);
    create_file(&result_str, &filename, Some(&buffer))?;
    Ok(key)
}

//return the content of a tree
pub fn get_tree(key: &str) -> io::Result<()>{
    let sub_dir_name: String = key.chars().take(2).collect();
    let filename: String = key.chars().skip(2).collect();
    let file_path: String = format!(".rit/objects/{}/{}", sub_dir_name, filename);
    let path = PathBuf::from(&file_path);
    let buffer = open_file(&path)?;

    let data = uncompress_data(&buffer)?;
    
    let cursor = Cursor::new(data);
    let reader = BufReader::new(cursor);

    for line in reader.lines() {
        match line {
            Ok(valid_line) => {
                // Parse the line 
                match parse(&valid_line) {
                    Ok(json_obj) => {
                        // Print out the tree data
                        if let (Some(file_type), Some(hashvalue), Some(filename)) = (
                            json_obj["type"].as_str(),
                            json_obj["hashvalue"].as_str(),
                            json_obj["filename"].as_str(),
                        ) {
                            println!("{} {} {}", file_type, hashvalue, filename);
                        } else {
                            println!("Could not retrieve one or more fields.");
                        }
                    }
                    Err(_e) => {
                        eprintln!("fatal: not a tree object");
                        return Ok(());
                    }
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
    Ok(())
}

//return the filename and hash if it exist, otherwise NULL
// pub fn update_tree(root_key: &str, full_path: &str, file_key: &str) -> io::Result<String> {
//     let path = Path::new(full_path);

//     let mut parent_key = root_key.to_string();
//     let file_name = path.file_name()
//         .and_then(|n| n.to_str())
//         .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File name not found"))?;

//     for component in path.components() {
//         let sub_dir_name: String = parent_key.chars().take(2).collect();
//         let filename: String = parent_key.chars().skip(2).collect();
//         let parent_file_path = format!(".rit/objects/{}/{}", sub_dir_name, filename);
//         let buffer = open_file(&parent_file_path)?;
//         let data = uncompress_data(&buffer)?;

//         let cursor = Cursor::new(data);
//         let reader = BufReader::new(cursor);

//         for line in reader.lines() {
//             match line {
//                 Ok(valid_line) => {
//                     match parse(&valid_line) {
//                         Ok(json_obj) => {
//                             if let (Some(_obj_file_type), Some(obj_hashvalue), Some(obj_filename)) = (
//                                 json_obj["type"].as_str(),
//                                 json_obj["hashvalue"].as_str(),
//                                 json_obj["filename"].as_str(),
//                             ) {
//                                 if obj_filename == file_name {
//                                     return Ok(obj_hashvalue.to_string());
//                                 } else if file_key ==  obj_hashvalue {
//                                     return Ok(obj_filename.to_string());
//                                 } else if component.as_os_str() == std::ffi::OsStr::new(obj_filename) {
//                                     parent_key = obj_hashvalue.to_string();
//                                 } else {
//                                     continue;
//                                 }
//                             }
//                         }
//                         Err(_e) => {
//                             return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a tree object"));
//                         }
//                     }
//                 }
//                 Err(e) => eprintln!("Error reading line: {}", e),
//             }
//         }
//     }
//     return Ok("NULL".to_string())
// }

//return the filename and hash if it exist, otherwise NULL
pub fn obj_in_tree(root_key: &str, full_path: &str, file_key: &str) -> io::Result<String> {
    let path = Path::new(full_path);

    let mut parent_key = root_key.to_string();
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File name not found"))?;

    for component in path.components() {
        let sub_dir_name: String = parent_key.chars().take(2).collect();
        let filename: String = parent_key.chars().skip(2).collect();
        let parent_file_path = format!(".rit/objects/{}/{}", sub_dir_name, filename);
        let path = PathBuf::from(&parent_file_path);
        let buffer = open_file(&path)?;
        let data = uncompress_data(&buffer)?;

        let cursor = Cursor::new(data);
        let reader = BufReader::new(cursor);

        for line in reader.lines() {
            match line {
                Ok(valid_line) => {
                    match parse(&valid_line) {
                        Ok(json_obj) => {
                            if let (Some(_obj_file_type), Some(obj_hashvalue), Some(obj_filename)) = (
                                json_obj["type"].as_str(),
                                json_obj["hashvalue"].as_str(),
                                json_obj["filename"].as_str(),
                            ) {
                                if obj_filename == file_name {
                                    return Ok(obj_hashvalue.to_string());
                                } else if file_key ==  obj_hashvalue {
                                    return Ok(obj_filename.to_string());
                                } else if component.as_os_str() == std::ffi::OsStr::new(obj_filename) {
                                    parent_key = obj_hashvalue.to_string();
                                } else {
                                    continue;
                                }
                            }
                        }
                        Err(_e) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a tree object"));
                        }
                    }
                }
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }
    }
    return Ok("NULL".to_string())
}


//return the content of a blob
pub fn get_data(key: &str) -> io::Result<()> {
    let sub_dir_name: String = key.chars().take(2).collect();
    let filename: String = key.chars().skip(2).collect();
    let file_path = format!(".rit/objects/{}/{}", sub_dir_name, filename);
    let path = PathBuf::from(&file_path);
    let buffer = open_file(&path)?;
    
    let data = uncompress_data(&buffer)?;

    let cursor = Cursor::new(data.clone());
    let reader = BufReader::new(cursor);

    if let Some(Ok(valid_line)) = reader.lines().next() {
        match parse(&valid_line) {
            Ok(json_obj) => {
                if let (Some(file_type), Some(hashvalue), Some(filename)) = (
                    json_obj["type"].as_str(),
                    json_obj["hashvalue"].as_str(),
                    json_obj["filename"].as_str(),
                ) {
                    println!("fatal: rit cat-file {}: bad file", key);
                    return Ok(());
                }
            }
            Err(_) => {
                if let Ok(string_data) = String::from_utf8(data) {
                    println!("{}", string_data);
                }
            }
        }
    } else {
        eprintln!("Error reading the first line.");
    }
    Ok(())
}